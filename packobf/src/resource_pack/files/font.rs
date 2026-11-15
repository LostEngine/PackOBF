use crate::minecraft::json::font::java_assets_font_Font;
use crate::resource_pack::identifier::Identifier;
use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Font {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(flatten)]
    pub json: java_assets_font_Font,
}

impl std::fmt::Display for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).map_err(|_| std::fmt::Error)?)
    }
}

impl Font {

    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut font: Self = serde_json::from_str(json)?;

        font.overlay = overlay.into();
        font.identifier = identifier;

        Ok(font)
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() { "".to_string() } else { format!("{}/", self.overlay) };
        format!("{}assets/{}/font/{}.json", prefix, self.identifier.namespace, self.identifier.path)
    }
}
