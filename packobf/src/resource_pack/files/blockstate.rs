use crate::minecraft::json::block_state_definition::java_assets_block_state_definition_BlockStateDefinition;
use crate::resource_pack::identifier::Identifier;
use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockstate {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(flatten)]
    pub json: java_assets_block_state_definition_BlockStateDefinition,
}

impl std::fmt::Display for Blockstate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).map_err(|_| std::fmt::Error)?)
    }
}

impl Blockstate {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut blockstate: Self = serde_json::from_str(json)?;

        blockstate.overlay = overlay.into();
        blockstate.identifier = identifier;

        Ok(blockstate)
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{}assets/{}/blockstates/{}.json",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
