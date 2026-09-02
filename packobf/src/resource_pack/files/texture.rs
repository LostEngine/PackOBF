use std::time::Duration;
use once_cell::sync::Lazy;
use oxipng::{indexset, optimize_from_memory, Deflater, FilterStrategy, PngError, StripChunks};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::UnboundedSender;
use crate::cache::{Cache, ItemType};
use crate::{profile_scope, LogLevel, LogMessage};
use crate::options::{Compression, Options, ULTRA_ZOPFLI_OPTIONS};
use crate::png::{crc, recoverer};
use crate::png::zopfli_png_idat_rewriter::rewrite_idat_with_zopfli;

#[derive(Clone, Debug)]
pub struct Texture {
    pub bytes: Vec<u8>,
}

impl Texture {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn optimize(
        &mut self,
        options: &Options,
        logger: &UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
        path: &str,
    ) {
        self.bytes =
            Self::cache_or_optimize(&self.bytes, options, logger, cache, path);
        if options.corrupt_png_files {
            match crc::modify_png_crcs(&self.bytes) {
                Ok(bytes) => {
                    self.bytes = bytes;
                }
                Err(e) => {
                    let _ = logger.send(LogMessage {
                        level: LogLevel::Warning,
                        message: format!(
                            "Could not corrupt image '{}'. Error: {}",
                            path,
                            e
                        ),
                    });
                }
            }
        }
    }

    fn cache_or_optimize(
        bytes: &[u8],
        options: &Options,
        logger: &UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
        path: &str,
    ) -> Vec<u8> {
        profile_scope!(std::any::type_name_of_val(&Self::cache_or_optimize));
        if let Some(cache) = cache {
            let mut sha256 = Sha256::new();
            sha256.update(bytes);
            let hash: [u8; 32] = sha256.finalize().into();

            if let Some(bytes) = cache
                .with_item(&hash, ItemType::Image, |it| {
                    (it.compression as u8 >= options.compression as u8)
                        .then(|| it.data.clone())
                })
                .flatten()
            {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Info,
                    message: format!("Image '{}' was loaded from cache.", path),
                });
                return bytes;
            }
        }

        let oxipng_options = match options.compression {
            Compression::Fastest => FASTEST_OPTIONS.clone(),
            Compression::Fast => FAST_OPTIONS.clone(),
            Compression::Normal => ANALYZE_OPTIONS.clone(),
            Compression::Best => ANALYZE_OPTIONS.clone(),
            Compression::Ultra => ULTRA_OPTIONS.clone(),
        };

        match optimize(bytes, &oxipng_options, &options.compression, logger) {
            Ok(value) => {
                match options.compression {
                    Compression::Normal | Compression::Best => {

                    }
                    _ => {}
                }
                if let Some(cache) = cache {
                    cache.add_item(
                        bytes,
                        &*value,
                        options.compression as u8,
                        ItemType::Image,
                    )
                }
                value
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Info,
                    message: format!(
                        "Could not optimize image '{}'. Trying to recover it. Error: {}",
                        path, e
                    ),
                });
                match recoverer::recover_png(bytes) {
                    Ok(value) => {
                        let _ = logger.send(LogMessage {
                            level: LogLevel::Info,
                            message: format!("Image '{}' was recovered successfully.", path),
                        });
                        match optimize(value.as_slice(), &oxipng_options, &options.compression, logger) {
                            Ok(value) => {
                                if let Some(cache) = cache {
                                    cache.add_item(
                                        bytes,
                                        &*value,
                                        options.compression as u8,
                                        ItemType::Image,
                                    )
                                }
                                value
                            }
                            Err(e) => {
                                let _ = logger.send(LogMessage {
                                    level: LogLevel::Warning,
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
                            level: LogLevel::Warning,
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

fn optimize(data: &[u8], opts: &oxipng::Options, compression: &Compression, logger: &UnboundedSender<LogMessage>) -> Result<Vec<u8>, PngError> {
    let mut data = optimize_from_memory(data, &opts)?;
    match compression {
        Compression::Normal | Compression::Best => {
            data = rewrite_idat_with_zopfli(data.as_slice(), compression, logger);
        }
        _ => {}
    }
    Ok(data)
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

/// Libdeflater (Level 9) is used to determine which zopfli options are going to be used.
/// See [options::analyze](crate::options::analyze).
static ANALYZE_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
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
    deflater: Deflater::Libdeflater { compression: 9 },
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
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
    deflater: Deflater::Zopfli(ULTRA_ZOPFLI_OPTIONS.to_owned()),
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});
//</editor-fold>

