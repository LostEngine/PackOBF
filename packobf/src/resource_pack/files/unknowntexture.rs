use std::num::NonZeroU64;
use std::time::Duration;
use once_cell::sync::Lazy;
use oxipng::{indexset, optimize_from_memory, Deflater, FilterStrategy, StripChunks};
use sha2::{Digest, Sha256};
use crate::cache::{Cache, ItemType};
use crate::LogLevel::{Info, Warning};
use crate::{options, profile_scope, LogMessage};
use crate::options::{Compression, Options};
use crate::png::{crc, recoverer};

#[derive(Clone, Debug)]
pub struct UnknownTexture {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// Decodes PNG dimensions and estimates uncompressed IDAT size instantly from raw headers.
fn get_raw_uncompressed_size(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 26 {
        return None;
    }
    if bytes[0..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }

    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?) as usize;
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?) as usize;
    let bit_depth = bytes[24] as usize;
    let color_type = bytes[25];

    let channels = match color_type {
        0 => 1, // Grayscale
        2 => 3, // RGB
        3 => 1, // Palette (indexed)
        4 => 2, // Grayscale + Alpha
        6 => 4, // RGBA
        _ => 4, // Fallback
    };

    let bits_per_pixel = channels * bit_depth;
    let row_bytes = (width * bits_per_pixel).div_ceil(8) + 1;
    Some(row_bytes * height)
}

/// Formula generated using a script
fn get_zopfli_options_normal(raw_size: usize) -> oxipng::ZopfliOptions {
    let (iterations, without_improvement, splits) = if raw_size < 10_000 {
        (10, 6, 1)
    } else if raw_size < 100_000 {
        (9, 3, 1)
    } else if raw_size < 1_000_000 {
        (9, 2, 1)
    } else {
        (9, 1, 2)
    };

    oxipng::ZopfliOptions {
        iteration_count: NonZeroU64::new(iterations).unwrap(),
        iterations_without_improvement: NonZeroU64::new(without_improvement).unwrap(),
        maximum_block_splits: splits,
    }
}

fn get_zopfli_options_best(raw_size: usize) -> oxipng::ZopfliOptions {
    let (iterations, without_improvement, splits) = if raw_size < 10_000 {
        (10, 6, 1)
    } else if raw_size < 100_000 {
        (16, 6, 1)
    } else if raw_size < 1_000_000 {
        (20, 6, 2)
    } else {
        (22, 7, 2)
    };

    oxipng::ZopfliOptions {
        iteration_count: NonZeroU64::new(iterations).unwrap(),
        iterations_without_improvement: NonZeroU64::new(without_improvement).unwrap(),
        maximum_block_splits: splits,
    }
}

impl UnknownTexture {
    pub fn new(path: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }

    pub fn optimize(
        &mut self,
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
    ) {
        self.bytes = Self::cache_or_optimize(&self.bytes, options, logger, cache, self.path.as_str());
        if options.corrupt_png_files {
            match crc::modify_png_crcs(&self.bytes) {
                Ok(bytes) => {
                    self.bytes = bytes;
                }
                Err(e) => {
                    let _ = logger.send(LogMessage {
                        level: Warning,
                        message: format!("Could not corrupt image '{}'. Error: {}", self.path.as_str(), e),
                    });
                }
            }
        }
    }

    fn cache_or_optimize(
        bytes: &[u8],
        options: &Options,
        logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
        path: &str,
    ) -> Vec<u8> {
        profile_scope!("cache_or_optimize::texture");
        if let Some(cache) = cache {
            let mut sha256 = Sha256::new();
            sha256.update(bytes);
            let hash: [u8; 32] = sha256.finalize().into();

            if let Some(bytes) = cache
                .with_item(&hash, ItemType::Image, |it| {
                    (it.compression as u8 >= options.compression.clone() as u8)
                        .then(|| it.data.clone())
                })
                .flatten()
            {
                let _ = logger.send(LogMessage {
                    level: Info,
                    message: format!("Image '{}' was loaded from cache.", path),
                });
                return bytes;
            }
        }

        let oxipng_options = match options.compression {
            Compression::Fastest => FASTEST_OPTIONS.clone(),
            Compression::Fast => FAST_OPTIONS.clone(),
            Compression::Normal => {
                let mut opts = NORMAL_OPTIONS.clone();
                let raw_size = get_raw_uncompressed_size(bytes).unwrap_or(bytes.len());
                opts.deflater = Deflater::Zopfli(get_zopfli_options_normal(raw_size));
                opts
            }
            Compression::Best => {
                let mut opts = BEST_OPTIONS.clone();
                let raw_size = get_raw_uncompressed_size(bytes).unwrap_or(bytes.len());
                opts.deflater = Deflater::Zopfli(get_zopfli_options_best(raw_size));
                opts
            }
            Compression::Ultra => {
                ULTRA_OPTIONS.clone()
            }
        };

        match optimize_from_memory(bytes, &oxipng_options) {
            Ok(value) => {
                if let Some(cache) = cache {
                    cache.add_item(
                        bytes,
                        &*value,
                        options.compression.clone() as u8,
                        ItemType::Image,
                    )
                }
                value
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Info,
                    message: format!(
                        "Could not optimize image '{}'. Trying to recover it. Error: {}",
                        path, e
                    ),
                });
                match recoverer::recover_png(bytes) {
                    Ok(value) => {
                        let _ = logger.send(LogMessage {
                            level: Info,
                            message: format!("Image '{}' was recovered successfully.", path),
                        });
                        match optimize_from_memory(value.as_slice(), &oxipng_options) {
                            Ok(value) => {
                                if let Some(cache) = cache {
                                    cache.add_item(
                                        bytes,
                                        &*value,
                                        options.compression.clone() as u8,
                                        ItemType::Image,
                                    )
                                }
                                value
                            }
                            Err(e) => {
                                let _ = logger.send(LogMessage {
                                    level: Warning,
                                    message: format!(
                                        "Could not optimize image '{}'. Skipping optimization. Error: {}",
                                        path,
                                        e),
                                });
                                value
                            }
                        }
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                            level: Warning,
                            message: format!(
                                "Could not recover image '{}'. Skipping optimization. Error: {}",
                                path, e
                            ),
                        });
                        bytes.to_owned()
                    }
                }
            }
        }
    }
}

// <editor-fold desc="Oxipng options" defaultstate="collapsed">
static FASTEST_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
    fix_errors: true,
    force: false,
    filters: indexset! {
        FilterStrategy::NONE,
        FilterStrategy::SUB,
        FilterStrategy::UP,
        FilterStrategy::AVERAGE,
        FilterStrategy::PAETH,
        FilterStrategy::MinSum,
        FilterStrategy::Entropy,
        FilterStrategy::Bigrams,
        FilterStrategy::BigEnt,
        FilterStrategy::Brute {
            num_lines: 8,
            level: 12,
        },
    },
    interlace: Some(false),
    optimize_alpha: true,
    bit_depth_reduction: true,
    color_type_reduction: true,
    palette_reduction: true,
    grayscale_reduction: true,
    idat_recoding: true,
    scale_16: false,
    strip: StripChunks::All,
    deflater: Deflater::Libdeflater { compression: 6 }, // 6: default compression level
    fast_evaluation: true,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});

static FAST_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
    fix_errors: true,
    force: false,
    filters: indexset! {
        FilterStrategy::NONE,
        FilterStrategy::SUB,
        FilterStrategy::UP,
        FilterStrategy::AVERAGE,
        FilterStrategy::PAETH,
        FilterStrategy::MinSum,
        FilterStrategy::Entropy,
        FilterStrategy::Bigrams,
        FilterStrategy::BigEnt,
        FilterStrategy::Brute {
            num_lines: 8,
            level: 12,
        },
    },
    interlace: Some(false),
    optimize_alpha: true,
    bit_depth_reduction: true,
    color_type_reduction: true,
    palette_reduction: true,
    grayscale_reduction: true,
    idat_recoding: true,
    scale_16: false,
    strip: StripChunks::All,
    deflater: Deflater::Libdeflater { compression: 12 }, // 12: max compression level for libdeflater
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});

static NORMAL_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
    fix_errors: true,
    force: false,
    filters: indexset! {
        FilterStrategy::NONE,
        FilterStrategy::SUB,
        FilterStrategy::UP,
        FilterStrategy::AVERAGE,
        FilterStrategy::PAETH,
        FilterStrategy::MinSum,
        FilterStrategy::Entropy,
        FilterStrategy::Bigrams,
        FilterStrategy::BigEnt,
        FilterStrategy::Brute {
            num_lines: 8,
            level: 12,
        },
    },
    interlace: Some(false),
    optimize_alpha: true,
    bit_depth_reduction: true,
    color_type_reduction: true,
    palette_reduction: true,
    grayscale_reduction: true,
    idat_recoding: true,
    scale_16: false,
    strip: StripChunks::All,
    deflater: Deflater::Zopfli(options::NORMAL_ZOPFLI_OPTIONS.to_owned()),
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});

static BEST_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
    fix_errors: true,
    force: false,
    filters: indexset! {
        FilterStrategy::NONE,
        FilterStrategy::SUB,
        FilterStrategy::UP,
        FilterStrategy::AVERAGE,
        FilterStrategy::PAETH,
        FilterStrategy::MinSum,
        FilterStrategy::Entropy,
        FilterStrategy::Bigrams,
        FilterStrategy::BigEnt,
        FilterStrategy::Brute {
            num_lines: 8,
            level: 12,
        },
    },
    interlace: Some(false),
    optimize_alpha: true,
    bit_depth_reduction: true,
    color_type_reduction: true,
    palette_reduction: true,
    grayscale_reduction: true,
    idat_recoding: true,
    scale_16: false,
    strip: StripChunks::All,
    deflater: Deflater::Zopfli(options::SLOW_ZOPFLI_OPTIONS.to_owned()),
    fast_evaluation: false,
    timeout: None,
    max_decompressed_size: None,
});

static ULTRA_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
    fix_errors: true,
    force: false,
    filters: indexset! {
        FilterStrategy::NONE,
        FilterStrategy::SUB,
        FilterStrategy::UP,
        FilterStrategy::AVERAGE,
        FilterStrategy::PAETH,
        FilterStrategy::MinSum,
        FilterStrategy::Entropy,
        FilterStrategy::Bigrams,
        FilterStrategy::BigEnt,
        FilterStrategy::Brute {
            num_lines: 8,
            level: 12,
        },
    },
    interlace: Some(false),
    optimize_alpha: true,
    bit_depth_reduction: true,
    color_type_reduction: true,
    palette_reduction: true,
    grayscale_reduction: true,
    idat_recoding: true,
    scale_16: false,
    strip: StripChunks::All,
    deflater: Deflater::Zopfli(options::ULTRA_ZOPFLI_OPTIONS.to_owned()),
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});
//</editor-fold>
