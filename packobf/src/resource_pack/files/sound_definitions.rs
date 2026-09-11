use crate::resource_pack::identifier::SoundId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::clean_json_numbers;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundDefinitions {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub namespace: String,

    #[serde(flatten)]
    pub variants: HashMap<String, SoundValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundValue {
    #[serde(default)]
    pub replace: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub sounds: Option<Vec<SoundInfo>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(from = "SoundInfoRaw", into = "SoundInfoRaw")]
pub enum SoundInfo {
    File {
        name: SoundId,
        volume: f32,
        pitch: f32,
        weight: u32,
        stream: bool,
        attenuation_distance: u32,
        preload: bool,
    },
    Event {
        name: String,
        volume: f32,
        pitch: f32,
        weight: u32,
        stream: bool,
        attenuation_distance: u32,
        preload: bool,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SoundInfoData {
    File {
        name: SoundId,
        #[serde(default = "default_one", skip_serializing_if = "is_one")]
        volume: f32,
        #[serde(default = "default_one", skip_serializing_if = "is_one")]
        pitch: f32,
        #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
        weight: u32,
        #[serde(default)]
        stream: bool,
        #[serde(default = "default_16_u32", skip_serializing_if = "is_16_u32")]
        attenuation_distance: u32,
        #[serde(default)]
        preload: bool,
    },
    Event {
        name: String,
        #[serde(default = "default_one", skip_serializing_if = "is_one")]
        volume: f32,
        #[serde(default = "default_one", skip_serializing_if = "is_one")]
        pitch: f32,
        #[serde(default = "default_one_u32", skip_serializing_if = "is_one_u32")]
        weight: u32,
        #[serde(default)]
        stream: bool,
        #[serde(default = "default_16_u32", skip_serializing_if = "is_16_u32")]
        attenuation_distance: u32,
        #[serde(default)]
        preload: bool,
        #[serde(rename = "type")]
        _type: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SoundInfoRaw {
    SimpleFile(SoundId),
    Full(SoundInfoData),
}

impl From<SoundInfoRaw> for SoundInfo {
    fn from(raw: SoundInfoRaw) -> Self {
        match raw {
            SoundInfoRaw::SimpleFile(id) => SoundInfo::File {
                name: id,
                volume: default_one(),
                pitch: default_one(),
                weight: default_one_u32(),
                stream: false,
                attenuation_distance: default_16_u32(),
                preload: false,
            },
            SoundInfoRaw::Full(data) => match data {
                SoundInfoData::File { name, volume, pitch, weight, stream, attenuation_distance, preload } =>
                    SoundInfo::File { name, volume, pitch, weight, stream, attenuation_distance, preload },
                SoundInfoData::Event { name, volume, pitch, weight, stream, attenuation_distance, preload, _type } =>
                    SoundInfo::Event { name, volume, pitch, weight, stream, attenuation_distance, preload },
            },
        }
    }
}

impl From<SoundInfo> for SoundInfoRaw {
    fn from(info: SoundInfo) -> Self {
        match info {
            SoundInfo::File { name, volume, pitch, weight, stream, attenuation_distance, preload }
            if is_one(&volume) && is_one(&pitch) && is_one_u32(&weight)
                && !stream && is_16_u32(&attenuation_distance) && !preload => {
                SoundInfoRaw::SimpleFile(name)
            }
            SoundInfo::File { name, volume, pitch, weight, stream, attenuation_distance, preload } => {
                SoundInfoRaw::Full(SoundInfoData::File { name, volume, pitch, weight, stream, attenuation_distance, preload })
            }
            SoundInfo::Event { name, volume, pitch, weight, stream, attenuation_distance, preload } => {
                SoundInfoRaw::Full(SoundInfoData::Event { name, volume, pitch, weight, stream, attenuation_distance, preload, _type: "event".to_string() })
            }
        }
    }
}

impl std::fmt::Display for SoundDefinitions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).map_err(|_| std::fmt::Error)?)
    }
}

impl SoundDefinitions {
    pub fn from_json(
        overlay: impl Into<String>,
        namespace: impl Into<String>,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut blockstate: SoundDefinitions = serde_json::from_str(json)?;

        blockstate.overlay = overlay.into();
        blockstate.namespace = namespace.into();

        Ok(blockstate)
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{}/", x),
        };
        format!("{}assets/{}/sounds.json", prefix, self.namespace)
    }
}

fn default_one() -> f32 {
    1.0
}

fn default_one_u32() -> u32 {
    1
}

fn default_16_u32() -> u32 {
    16
}

fn is_one(f: &f32) -> bool {
    (*f - 1.0).abs() < f32::EPSILON
}

fn is_one_u32(f: &u32) -> bool {
    f == &1
}

fn is_16_u32(f: &u32) -> bool {
    f == &16
}
