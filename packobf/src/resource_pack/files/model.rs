use crate::resource_pack::identifier::{Identifier, ModelId, TextureId};
use crate::utils::clean_json_numbers;
use crate::version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// models are referenced in `items` and `blockstates`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ModelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambientocclusion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<HashMap<String, Display>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<HashMap<String, TextureId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<Element>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<Override>>,
    #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
    pub gui_light: Option<String>,
}

impl Model {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut model: Model = serde_json::from_str(json)?;

        model.overlay = overlay.into();
        model.identifier = identifier;

        Ok(model)
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() {
            "".to_string()
        } else {
            format!("{}/", self.overlay)
        };
        format!(
            "{}assets/{}/models/{}.json",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(
            f,
            "{}",
            serde_json::to_string(&val).map_err(|_| std::fmt::Error)?
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Display {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Element {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shade: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_emission: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub faces: Option<HashMap<String, Face>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f32>,

    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub origin: Option<[f32; 3]>,

    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub rescale: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Face {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv: Option<[f32; 4]>,

    pub texture: String, // an id to textures in the model

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cullface: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tintindex: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Override {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicate: Option<Vec<String>>,
    pub model: String,
}

pub fn is_none_or_newer_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_newer_than_26_1(&())
}

pub fn is_none_or_older_than_1_21_11<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_older_than_1_21_11(&())
}
