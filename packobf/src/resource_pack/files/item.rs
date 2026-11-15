use crate::minecraft::json::item_definition::java_assets_item_definition_ItemDefinition;
use crate::resource_pack::identifier::Identifier;
use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(flatten)]
    pub json: java_assets_item_definition_ItemDefinition,
}

impl std::fmt::Display for Item {
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

impl Item {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut item: Self = serde_json::from_str(json)?;

        item.overlay = overlay.into();
        item.identifier = identifier;

        Ok(item)
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{}assets/{}/items/{}.json",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
