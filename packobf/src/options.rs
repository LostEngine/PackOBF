use once_cell::sync::Lazy;
use std::num::NonZeroU64;
use clap::{Parser, ValueEnum};

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
}

#[derive(ValueEnum, Clone, Debug)]
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
            shader_compression: ShaderCompression::None,
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
#[derive(ValueEnum, Clone, Debug)]
pub enum Compression {
    Simplest = 0,
    Normal = 1,
    Max = 2,
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug)]
#[derive(PartialEq)]
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
