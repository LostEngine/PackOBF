use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::resource_pack::identifier::TextureIdWithExt;
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaAssetsFontGlyphProviderTtfSkip {
    String1(String),
    List22(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_font_Font {
    pub providers: Vec<java_assets_font_GlyphProvider>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_font_FontOption {
    #[serde(rename = "uniform", alias = "minecraft:uniform")]
    Uniform,
    #[serde(rename = "jp", alias = "minecraft:jp")]
    JapaneseVariants,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum java_assets_font_GlyphProvider {
    #[serde(rename = "bitmap", alias = "minecraft:bitmap")]
    Bitmap {
        file: TextureIdWithExt,
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        height: Option<i32>,
        #[serde(deserialize_with = "deserialize_i32")]
        ascent: i32,
        chars: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<HashMap<java_assets_font_FontOption, bool>>,
    },
    #[serde(rename = "reference", alias = "minecraft:reference")]
    Reference {
        /// Identifier for the `font` registry.
        id: crate::resource_pack::identifier::Identifier,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<HashMap<java_assets_font_FontOption, bool>>,
    },
    #[serde(rename = "space", alias = "minecraft:space")]
    Space {
        advances: HashMap<String, f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<HashMap<java_assets_font_FontOption, bool>>,
    },
    #[serde(rename = "ttf", alias = "minecraft:ttf")]
    Ttf {
        file: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oversample: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        shift: Option<[f32; 2]>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skip: Option<JavaAssetsFontGlyphProviderTtfSkip>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<HashMap<java_assets_font_FontOption, bool>>,
    },
    #[serde(rename = "unihex", alias = "minecraft:unihex")]
    Unihex {
        /// ZIP archive containing one or more *.hex files (files in archive with different extensions are ignored).
        hex_file: String,
        /// List of ranges to override the size of.
        #[serde(skip_serializing_if = "Option::is_none")]
        size_overrides: Option<Vec<java_assets_font_UnihexOverrideRange>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<HashMap<java_assets_font_FontOption, bool>>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_font_UnihexOverrideRange {
    /// Minimum in codepoint range (inclusive).
    pub from: String,
    /// Maximum in codepoint range (inclusive).
    pub to: String,
    /// Position of left-most column of the glyph.
    pub left: u8,
    /// Position of right-most column of the glyph.
    pub right: u8,
}
