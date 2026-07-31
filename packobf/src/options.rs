use once_cell::sync::Lazy;
use std::num::NonZeroU64;
use clap::{Parser, ValueEnum};
use libdeflater::{CompressionLvl, Compressor};

#[derive(Parser, Clone, Debug)]
#[group(id = "options")]
pub struct Options {
    #[arg(short, long, value_enum, default_value_t = Compression::Normal)]
    pub compression: Compression,
    #[arg(long, value_enum, default_value_t = ShaderCompression::Minify)]
    pub shader_compression: ShaderCompression,
    #[arg(long)]
    pub rename_files: bool,
    #[arg(long)]
    pub block_unzipping: bool,
    #[arg(long)]
    pub corrupt_png_files: bool,
    #[arg(long)]
    pub num_threads: Option<usize>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Preset {
    Fastest,
    Fast,
    Normal,
    Best,
}

impl Options {
    pub fn fastest() -> Self {
        Self {
            compression: Compression::Fastest,
            shader_compression: ShaderCompression::None,
            rename_files: false,
            block_unzipping: false,
            corrupt_png_files: false,
            num_threads: None,
        }
    }

    pub fn fast() -> Self {
        Self {
            compression: Compression::Fast,
            shader_compression: ShaderCompression::None,
            rename_files: false,
            block_unzipping: false,
            corrupt_png_files: false,
            num_threads: None,
        }
    }

    pub fn normal() -> Self {
        Self {
            compression: Compression::Normal,
            shader_compression: ShaderCompression::None,
            rename_files: false,
            block_unzipping: false,
            corrupt_png_files: false,
            num_threads: None,
        }
    }

    pub fn best() -> Self {
        Self {
            compression: Compression::Best,
            shader_compression: ShaderCompression::MinifyAndObfuscate,
            rename_files: true,
            block_unzipping: true,
            corrupt_png_files: true,
            num_threads: None,
        }
    }

    pub fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::Fastest => Self::fastest(),
            Preset::Fast => Self::fast(),
            Preset::Normal => Self::normal(),
            Preset::Best => Self::best(),
        }
    }
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug)]
pub enum Compression {
    Fastest = 0,
    Fast = 1,
    Normal = 2,
    Best = 3,
    Ultra = 4,
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug)]
#[derive(PartialEq)]
pub enum ShaderCompression {
    None = 0,
    Minify = 1,
    MinifyAndObfuscate = 2,
}

#[allow(clippy::unwrap_used)]
pub static ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(25).unwrap(),
    iterations_without_improvement: NonZeroU64::new(3).unwrap(),
    maximum_block_splits: 15,
});

#[allow(clippy::unwrap_used)]
pub static FASTEST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(3).unwrap(),
    iterations_without_improvement: NonZeroU64::new(1).unwrap(),
    maximum_block_splits: 2,
});

#[allow(clippy::unwrap_used)]
pub static FAST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(5).unwrap(),
    iterations_without_improvement: NonZeroU64::new(2).unwrap(),
    maximum_block_splits: 5,
});

#[allow(clippy::unwrap_used)]
pub static NORMAL_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(12).unwrap(),
    iterations_without_improvement: NonZeroU64::new(2).unwrap(),
    maximum_block_splits: 10,
});

#[allow(clippy::unwrap_used)]
pub static SLOW_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(20).unwrap(),
    iterations_without_improvement: NonZeroU64::new(3).unwrap(),
    maximum_block_splits: 15,
});

#[allow(clippy::unwrap_used)]
pub static SLOWEST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(25).unwrap(),
    iterations_without_improvement: NonZeroU64::new(3).unwrap(),
    maximum_block_splits: 15,
});

#[allow(clippy::unwrap_used)]
pub static ULTRA_ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(40).unwrap(),
    iterations_without_improvement: NonZeroU64::new(40).unwrap(),
    maximum_block_splits: 25,
});

pub enum PreCheckResult {
    /// Skip Zopfli entirely
    Skip,
    /// Use Zopfli with dynamically assigned options
    CompressWithZopfli(zopfli::Options),
}

/// Pre-checks data compressibility using libdeflater (Level 9)
/// and dynamically calculates the Zopfli config for 'normal' preset.
pub fn analyze_and_get_zopfli_config_normal(data: &[u8]) -> PreCheckResult {
    let (original_size, savings_ratio) = match analyze(data) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // Less than 1% savings
    if savings_ratio < 0.01 {
        return PreCheckResult::Skip; // Don't waste CPU time on Zopfli
    }

    // 1% to 8% savings
    if savings_ratio < 0.08 {
        return PreCheckResult::CompressWithZopfli(FASTEST_ZOPFLI_OPTIONS.to_owned());
    }

    // > 8% savings
    PreCheckResult::CompressWithZopfli(match original_size {
        0..=51_200 => SLOW_ZOPFLI_OPTIONS.to_owned(),

        51_201..=512_000 => NORMAL_ZOPFLI_OPTIONS.to_owned(),

        _ => FAST_ZOPFLI_OPTIONS.to_owned(),
    })
}

/// Pre-checks data compressibility using libdeflater (Level 9)
/// and dynamically calculates the Zopfli config for 'best' preset.
pub fn analyze_and_get_zopfli_config_best(data: &[u8]) -> PreCheckResult {
    let (original_size, savings_ratio) = match analyze(data) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // Less than 1% savings
    if savings_ratio < 0.01 {
        return PreCheckResult::Skip; // Don't waste CPU time on Zopfli
    }

    // 1% to 8% savings
    if savings_ratio < 0.08 {
        return PreCheckResult::CompressWithZopfli(FAST_ZOPFLI_OPTIONS.to_owned());
    }

    // > 8% savings
    PreCheckResult::CompressWithZopfli(match original_size {
        0..=51_200 => SLOWEST_ZOPFLI_OPTIONS.to_owned(),

        51_201..=512_000 => SLOW_ZOPFLI_OPTIONS.to_owned(),

        _ => NORMAL_ZOPFLI_OPTIONS.to_owned(),
    })
}

fn analyze(data: &[u8]) -> Result<(usize, f64), PreCheckResult> {
    let original_size = data.len();
    if original_size == 0 {
        return Err(PreCheckResult::Skip);
    }

    let mut compressor = Compressor::new(CompressionLvl::new(9).unwrap());
    let max_buf_len = compressor.deflate_compress_bound(original_size);
    let mut compressed_buf = vec![0u8; max_buf_len];

    let fast_compressed_size = match compressor.deflate_compress(data, &mut compressed_buf) {
        Ok(sz) => sz,
        Err(_) => return Err(PreCheckResult::Skip),
    };

    let bytes_saved = original_size.saturating_sub(fast_compressed_size);
    let savings_ratio = bytes_saved as f64 / original_size as f64;
    Ok((original_size, savings_ratio))
}
