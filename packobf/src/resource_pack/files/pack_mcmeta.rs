use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackMcmeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlays: Option<Overlay>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Overlay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<OverlayEntry>>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverlayEntry {
    pub directory: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_format: Option<PackVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_format: Option<PackVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<FormatRange>,

    #[serde(flatten)]
    extra: serde_json::Value,
}

// Versions are now decimal numbers, we can use either integer or two integers
// for the number before the decimal point and the number after the decimal point.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackVersion {
    Integer(i32),
    Decimal([i32; 2]),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormatRange {
    Int(i32),
    List([i32; 2]),
    Object {
        min_inclusive: PackVersion,
        max_inclusive: PackVersion,

        #[serde(flatten)]
        extra: serde_json::Value,
    },
}

impl std::fmt::Display for PackMcmeta {
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

impl PackMcmeta {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn path(&self) -> &'static str {
        "pack.mcmeta"
    }
}
