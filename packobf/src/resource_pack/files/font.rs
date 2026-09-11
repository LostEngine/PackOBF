use crate::resource_pack::identifier::{Identifier, TextureIdWithExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::clean_json_numbers;
use crate::version;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Font {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    pub providers: Vec<FontProvider>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FontProvider {
    #[serde(rename = "bitmap")]
    Bitmap {
        file: TextureIdWithExt,
        #[serde(default = "default_ascent")]
        ascent: i32,
        #[serde(default = "default_height")]
        height: i32,
        chars: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<FontFilter>,
    },

    #[serde(rename = "space")]
    Space {
        advances: HashMap<String, i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<FontFilter>,
    },

    #[serde(rename = "ttf")]
    Ttf {
        file: Identifier,
        #[serde(default = "default_size")]
        size: f32,
        #[serde(default = "default_oversample")]
        oversample: f32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        shift: Vec<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skip: Option<TtfSkip>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<FontFilter>,
    },

    #[serde(rename = "unihex")]
    Unihex {
        hex_file: String,
        #[serde(default, skip_serializing_if = "is_empty_or_older_than_26_1")]
        size_overrides: Vec<HexSizeOverride>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        filter: Option<FontFilter>,
    },

    #[serde(rename = "reference")]
    Reference {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filter: Option<FontFilter>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniform: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jp: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HexSizeOverride {
    pub from: String,
    pub to: String,
    pub left: i32,
    pub right: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TtfSkip {
    Single(String),
    Multiple(Vec<String>),
}

impl std::fmt::Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).map_err(|_| std::fmt::Error)?)
    }
}

fn default_ascent() -> i32 { 7 }
fn default_height() -> i32 { 8 }
fn default_size() -> f32 { 11.0 }
fn default_oversample() -> f32 { 1.5 }
pub fn is_none_or_older_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_older_than_26_1(&())
}
pub fn is_empty_or_older_than_26_1<T>(value: &[T]) -> bool {
    value.is_empty() || version::is_older_than_26_1(&())
}

impl Font {

    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut font: Font = serde_json::from_str(json)?;

        font.overlay = overlay.into();
        font.identifier = identifier;

        Ok(font)
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() { "".to_string() } else { format!("{}/", self.overlay) };
        format!("{}assets/{}/font/{}.json", prefix, self.identifier.namespace, self.identifier.path)
    }
}
