use crate::version::MinecraftVersion;
use clap::{Parser, ValueEnum};
use once_cell::sync::Lazy;
use std::num::NonZeroU64;

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
    #[arg(long)]
    pub target_version: Option<MinecraftVersion>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Preset {
    Fastest,
    Fast,
    Normal,
    Best,
    Ultra,
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
            target_version: None,
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
            target_version: None,
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
            target_version: None,
        }
    }

    pub fn best() -> Self {
        Self {
            compression: Compression::Best,
            shader_compression: ShaderCompression::None,
            rename_files: true,
            block_unzipping: true,
            corrupt_png_files: true,
            num_threads: None,
            target_version: None,
        }
    }

    pub fn ultra() -> Self {
        Self {
            compression: Compression::Ultra,
            shader_compression: ShaderCompression::MinifyAndObfuscate,
            rename_files: true,
            block_unzipping: true,
            corrupt_png_files: true,
            num_threads: None,
            target_version: None,
        }
    }

    pub fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::Fastest => Self::fastest(),
            Preset::Fast => Self::fast(),
            Preset::Normal => Self::normal(),
            Preset::Best => Self::best(),
            Preset::Ultra => Self::ultra(),
        }
    }
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Compression {
    Fastest = 0,
    Fast = 1,
    Normal = 2,
    Best = 3,
    Ultra = 4,
}

impl Compression {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Compression::Fastest,
            1 => Compression::Fast,
            2 => Compression::Normal,
            3 => Compression::Best,
            4 => Compression::Ultra,
            _ => Compression::Normal,
        }
    }
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum ShaderCompression {
    None = 0,
    Minify = 1,
    MinifyAndObfuscate = 2,
}

pub static ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| create_zopfli_options(25, 3, 15));

pub static FASTEST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(3, 1, 2));

pub static FAST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(5, 2, 5));

pub static NORMAL_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(12, 2, 10));

pub static SLOW_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(20, 3, 15));

pub static SLOWEST_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(25, 3, 15));

pub static ULTRA_ZOPFLI_OPTIONS: Lazy<zopfli::Options> =
    Lazy::new(|| create_zopfli_options(40, 40, 25));

#[allow(clippy::unwrap_used)]
fn create_zopfli_options(
    iteration_count: u64,
    iterations_without_improvement: u64,
    maximum_block_splits: u16,
) -> zopfli::Options {
    zopfli::Options {
        iteration_count: NonZeroU64::new(iteration_count).unwrap(),
        iterations_without_improvement: NonZeroU64::new(iterations_without_improvement).unwrap(),
        maximum_block_splits,
    }
}

