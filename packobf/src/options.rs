use crate::version::MinecraftVersion;
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
    #[arg(long)]
    pub num_threads: Option<usize>,
    #[arg(long)]
    pub target_version: Option<MinecraftVersion>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Preset {
    Fast,
    Normal,
    Best,
}

impl Options {
    pub const fn fast() -> Self {
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

    pub const fn normal() -> Self {
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

    pub const fn best() -> Self {
        Self {
            compression: Compression::Normal,
            shader_compression: ShaderCompression::None,
            rename_files: true,
            block_unzipping: true,
            corrupt_png_files: true,
            num_threads: None,
            target_version: None,
        }
    }

    pub const fn from_preset(preset: Preset) -> Self {
        match preset {
            Preset::Fast => Self::fast(),
            Preset::Normal => Self::normal(),
            Preset::Best => Self::best(),
        }
    }
}

#[repr(u8)]
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum Compression {
    Fast = 0,
    Normal = 1,
}

impl Compression {
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Fast,
            _ => Self::Normal,
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
