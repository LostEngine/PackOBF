use crate::version::MinecraftVersion;
use clap::{Parser, ValueEnum};

#[derive(Parser, Clone, Debug)]
#[group(id = "options")]
pub struct Options {
    /// -c/--compression
    #[arg(short('c'), long, value_enum, default_value_t = Compression::Normal)]
    pub compression: Compression,
    /// -S/--shader-compression
    #[arg(short('S'), long, value_enum, default_value_t = ShaderCompression::None)]
    pub shader_compression: ShaderCompression,
    /// -s/--sound-quality
    #[arg(short('s'), long, value_enum, allow_negative_numbers = true, default_value_t = SoundQuality::Dynamic)]
    pub sound_quality: SoundQuality,
    /// -r/--rename-files
    #[arg(short('r'), long)]
    pub rename_files: bool,
    /// -b/--block-unzipping
    #[arg(short('b'), long)]
    pub block_unzipping: bool,
    /// -C/--corrupt-png-files
    #[arg(short('C'), long)]
    pub corrupt_png_files: bool,
    /// -n/num_threads
    #[arg(short('n') ,long)]
    pub num_threads: Option<usize>,
    /// -t/--target-version
    #[arg(short('t'), long)]
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
            sound_quality: SoundQuality::Dynamic,
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
            sound_quality: SoundQuality::Dynamic,
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
            sound_quality: SoundQuality::Dynamic,
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

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum SoundQuality {
    Dynamic,
    #[value(name = "-1")]
    MinusOne,
    #[value(name = "0")]
    Zero,
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
    #[value(name = "3")]
    Three,
    #[value(name = "4")]
    Four,
    #[value(name = "5")]
    Five,
    #[value(name = "6")]
    Six,
    #[value(name = "7")]
    Seven,
    #[value(name = "8")]
    Eight,
    #[value(name = "9")]
    Nine,
    #[value(name = "10")]
    Ten,
}

impl SoundQuality {
    pub const fn value(&self) -> Option<f32> {
        match self {
            Self::Dynamic => None,
            Self::MinusOne => Some(-0.1),
            Self::Zero => Some(0.0),
            Self::One => Some(0.1),
            Self::Two => Some(0.2),
            Self::Three => Some(0.3),
            Self::Four => Some(0.4),
            Self::Five => Some(0.5),
            Self::Six => Some(0.6),
            Self::Seven => Some(0.7),
            Self::Eight => Some(0.8),
            Self::Nine => Some(0.9),
            Self::Ten => Some(1.0),
        }
    }

    pub(crate) fn dynamic_quality(duration: f64) -> f32 {
        match duration {
            d if d < 2.0 => 0.1,
            d if d < 10.0 => 0.2,
            d if d < 120.0 => 0.3,
            _ => 0.4,
        }
    }
}
