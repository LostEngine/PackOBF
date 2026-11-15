use crate::minecraft::json::model::java_assets_model_Model;
use crate::resource_pack::identifier::Identifier;
use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};
use crate::resource_pack::mapping::get_mappings;

// models are referenced in `items` and `blockstates`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(flatten)]
    pub json: java_assets_model_Model,
}

impl Model {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut model: Self = serde_json::from_str(json)?;

        model.overlay = overlay.into();
        model.identifier = identifier;

        Ok(model)
    }

    pub fn path(&self) -> String {
        let mapped = get_mappings().model_mappings.get(&self.identifier.to_string()).map_or_else(|| self.identifier.clone(), |id| Identifier::parse(id));
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{}assets/{}/models/{}.json",
            prefix, mapped.namespace, mapped.path
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
