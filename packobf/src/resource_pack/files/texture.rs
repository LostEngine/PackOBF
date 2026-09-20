use crate::cache::{Cache, ItemType};
use crate::deflate::libdeflater::libdeflater_zlib;
use crate::options::{Compression, Options};
use crate::{profile_scope, LogLevel, LogMessage};
use once_cell::sync::Lazy;
use oxipng::{indexset, optimize_from_memory, Deflater, FilterStrategy, PngError, StripChunks};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
pub struct Texture {
    pub bytes: Vec<u8>,
}

impl Texture {
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn optimize(
        self,
        options: &Options,
        logger: &UnboundedSender<LogMessage>,
        cache: &Option<Cache>,
        path: &str,
    ) -> Vec<u8> {
        profile_scope!(std::any::type_name_of_val(&Self::optimize));

        let cache_data: Option<(&Cache, [u8; 32])> = cache.as_ref().map(|c| {
            let mut sha256 = Sha256::new();
            sha256.update(&self.bytes);
            let hash = sha256.finalize().into();
            (c, hash)
        });
        let compression_value = CompressionValue::new(options.compression, options.corrupt_png_files);
        if let Some(cache_data) = cache_data {
            if let Some(bytes) = cache_data.0
                .with_item(&cache_data.1, ItemType::Image, |it| {
                    compression_value.is_compatible_with(CompressionValue::from(it.compression))
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

        let mut oxipng_options = OPTIONS.clone();
        oxipng_options.disable_checksums = options.corrupt_png_files;
        oxipng_options.deflater = match options.compression {
            Compression::Fast => Deflater::Custom(|input, disable_checksums| {
                libdeflater_zlib(input, 6, !disable_checksums).map_err(|_| PngError::InvalidData)
            }),
            Compression::Normal => Deflater::Custom(|input, disable_checksums| {
                libdeflater_zlib(input, 12, !disable_checksums).map_err(|_| PngError::InvalidData)
            }),
        };

        match optimize_from_memory(&self.bytes, &oxipng_options) {
            Ok(value) => {
                if let Some(cache_data) = cache_data {
                    cache_data.0.add_item_hash(
                        cache_data.1,
                        &*value,
                        compression_value.into(),
                        ItemType::Image,
                    );
                }
                value
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Info,
                    message: format!(
                        "Could not optimize image '{}'. Skipping optimization. Error: {}",
                        path, e
                    ),
                });
                self.bytes
            }
        }
    }
}

struct CompressionValue {
    compression: Compression,
    corrupt_png_files: bool,
}

impl From<u8> for CompressionValue {
    fn from(value: u8) -> Self {
        let corrupt_png_files = value & 0b0001_0000 != 0;
        let compression = Compression::from_u8(value & 0b0000_1111);
        Self::new(compression, corrupt_png_files)
    }
}

impl From<CompressionValue> for u8 {
    fn from(value: CompressionValue) -> u8 {
        value.u8()
    }
}

impl CompressionValue {
    const fn new(compression: Compression, corrupt_png_files: bool) -> Self {
        Self { compression, corrupt_png_files }
    }

    const fn u8(&self) -> u8 {
        let mut value = self.compression as u8;
        if self.corrupt_png_files {
            value |= 0b0001_0000;
        }
        value
    }

    const fn is_compatible_with(&self, other: Self) -> bool {
        self.corrupt_png_files == other.corrupt_png_files && self.compression as u8 <= other.compression as u8
    }

}

static OPTIONS: Lazy<oxipng::Options> = Lazy::new(|| oxipng::Options {
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
    deflater: Deflater::Libdeflater {compression: 6},
    fast_evaluation: true,
    timeout: None,
    max_decompressed_size: None,
    disable_checksums: false,
});
