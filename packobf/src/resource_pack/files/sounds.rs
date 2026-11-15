use crate::minecraft::json::sounds::java_assets_sounds_Sounds;
use crate::utils::clean_json_numbers;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundDefinitions {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub namespace: String,

    #[serde(flatten)]
    pub json: java_assets_sounds_Sounds,
}

impl std::fmt::Display for SoundDefinitions {
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

impl SoundDefinitions {
    pub fn from_json(
        overlay: impl Into<String>,
        namespace: impl Into<String>,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut blockstate: Self = serde_json::from_str(json)?;

        blockstate.overlay = overlay.into();
        blockstate.namespace = namespace.into();

        Ok(blockstate)
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!("{}assets/{}/sounds.json", prefix, self.namespace)
    }
}
