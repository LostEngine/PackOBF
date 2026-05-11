use once_cell::sync::Lazy;
use std::num::NonZeroU64;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Options {
    pub compression: Compression,
    pub shader_compression: ShaderCompression,
    pub rename_files: bool,
    pub block_unzipping: bool,
    pub corrupt_png_files: bool,
}

#[derive(Clone, Debug)]
pub enum Preset {
    Simplest,
    Normal,
    Max,
}

impl Options {
    pub fn simplest() -> Self {
        Self {
            compression: Compression::Simplest,
            shader_compression: ShaderCompression::None,
            rename_files: false,
            block_unzipping: false,
            corrupt_png_files: false,
        }
    }

    pub fn normal() -> Self {
        Self {
            compression: Compression::Normal,
            shader_compression: ShaderCompression::Minify,
            rename_files: false,
            block_unzipping: false,
            corrupt_png_files: false,
        }
    }

    pub fn max() -> Self {
        Self {
            compression: Compression::Max,
            shader_compression: ShaderCompression::MinifyAndObfuscate,
            rename_files: true,
            block_unzipping: true,
            corrupt_png_files: true,
        }
    }

    pub fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::Simplest => Self::simplest(),
            Preset::Normal => Self::normal(),
            Preset::Max => Self::max(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Deserialize)]
pub enum Compression {
    Simplest = 0,
    Normal = 1,
    Max = 2,
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum ShaderCompression {
    None = 0,
    Minify = 1,
    MinifyAndObfuscate = 2,
}

pub static ZOPFLI_OPTIONS: Lazy<zopfli::Options> = Lazy::new(|| zopfli::Options {
    iteration_count: NonZeroU64::new(25).unwrap(),
    iterations_without_improvement: NonZeroU64::new(7).unwrap(),
    maximum_block_splits: 50,
});
