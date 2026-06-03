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
            Compression::Simplest => &DEFAULT_OPTIONS,
            Compression::Normal => &NORMAL_OPTIONS,
            Compression::Max => &MAX_OPTIONS,
        };
        match optimize_from_memory(bytes, oxipng_options) {
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
                        match optimize_from_memory(value.as_slice(), oxipng_options) {
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

/**
Currently, only the `deflater` option changes, but some other options could be modified based on the compression wanted.
*/
//<editor-fold desc="Oxipng options" defaultstate="collapsed">
static DEFAULT_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
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
    deflater: Deflater::Libdeflater { compression: 12 }, // 12: max compression level for libdeflater
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});

static MAX_OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
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
    deflater: Deflater::Zopfli(options::ZOPFLI_OPTIONS.to_owned()), // zopfli: best compression
    fast_evaluation: false,
    timeout: Some(Duration::from_secs(3)),
    max_decompressed_size: None,
});
//</editor-fold>

