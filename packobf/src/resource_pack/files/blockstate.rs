use crate::resource_pack::identifier::{Identifier, ModelId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::clean_json_numbers;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockstate {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, VariantValue>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub multipart: Option<Vec<MultipartCase>>,
}

/// A variant can be a single model object or a list of weighted model objects
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariantValue {
    Single(BlockModel),
    Multiple(Vec<BlockModel>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockModel {
    pub model: ModelId,

    #[serde(default, skip_serializing_if = "is_zero")]
    pub x: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub y: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub z: i32,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub uvlock: bool,

    #[serde(default = "default_weight", skip_serializing_if = "is_default_weight")]
    pub weight: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultipartCase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<MultipartCondition>,
    pub apply: VariantValue,
}

/// Logic for the 'when' block in multipart
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultipartCondition {
    /// matches: {"OR": [{"state": "val"}, {"state": "val2"}]}
    Or {
        #[serde(rename = "OR")]
        or: Vec<HashMap<String, String>>,
    },
    /// matches: {"AND": [{"state": "val"}, {"state": "val2"}]}
    And {
        #[serde(rename = "AND")]
        and: Vec<HashMap<String, String>>,
    },
    /// matches: {"state": "value|value2"}
    Single(HashMap<String, String>),
}

impl std::fmt::Display for Blockstate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).unwrap())
    }
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}
fn default_weight() -> i32 {
    1
}
fn is_default_weight(v: &i32) -> bool {
    *v == 1
}

impl Blockstate {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut blockstate: Blockstate = serde_json::from_str(json)?;

        blockstate.overlay = overlay.into();
        blockstate.identifier = identifier;

        Ok(blockstate)
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() {
            "".to_string()
        } else {
            format!("{}/", self.overlay)
        };
        format!(
            "{}assets/{}/blockstates/{}.json",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
