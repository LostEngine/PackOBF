use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use serde::de::Error;
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaAssetsSoundsSoundEventRegistrationSounds {
    /// Identifier for the `sound` registry.
    String1(crate::resource_pack::identifier::SoundId),
    java_assets_sounds_Sound2(java_assets_sounds_Sound),
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum java_assets_sounds_Sound {
    SoundEvent {
        /// Identifier for the `weighed_sound_event` registry.
        name: crate::resource_pack::identifier::Identifier,
        /// Defaults to 1.0.
        #[serde(skip_serializing_if = "Option::is_none")]
        volume: Option<f32>,
        /// Default is 1.0.
        #[serde(skip_serializing_if = "Option::is_none")]
        pitch: Option<f32>,
        /// Chance that this sound is selected to play. Defaults to 1.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        weight: Option<i32>,
        /// Whether the sound should be loaded when loading the pack instead of when the sound is played. Used by the underwater ambience. Defaults to false.
        #[serde(skip_serializing_if = "Option::is_none")]
        preload: Option<bool>,
        /// If true it will be streamed from its file. Sounds longer than a few seconds should enable this to avoid lag. Defaults to false.
        /// When false many instances of the sound can be ran at the same time. When true only allows 4 instances (of that type) can be played.
        #[serde(skip_serializing_if = "Option::is_none")]
        stream: Option<bool>,
        /// Modify sound reduction rate based on distance. Defaults to 16.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        attenuation_distance: Option<i32>,
        /// Custom field
        #[serde(rename = "type")]
        r#type: String,
    },
    File {
        /// Identifier for the `sound` registry.
        name: crate::resource_pack::identifier::SoundId,
        /// Defaults to 1.0.
        #[serde(skip_serializing_if = "Option::is_none")]
        volume: Option<f32>,
        /// Default is 1.0.
        #[serde(skip_serializing_if = "Option::is_none")]
        pitch: Option<f32>,
        /// Chance that this sound is selected to play. Defaults to 1.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        weight: Option<i32>,
        /// Whether the sound should be loaded when loading the pack instead of when the sound is played. Used by the underwater ambience. Defaults to false.
        #[serde(skip_serializing_if = "Option::is_none")]
        preload: Option<bool>,
        /// If true it will be streamed from its file. Sounds longer than a few seconds should enable this to avoid lag. Defaults to false.
        /// When false many instances of the sound can be ran at the same time. When true only allows 4 instances (of that type) can be played.
        #[serde(skip_serializing_if = "Option::is_none")]
        stream: Option<bool>,
        /// Modify sound reduction rate based on distance. Defaults to 16.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        attenuation_distance: Option<i32>,
    },
}

impl<'de> Deserialize<'de> for java_assets_sounds_Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value =
            serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::Object(ref mut map) = value
            && !map.contains_key("type") {
                map.insert(
                    "type".to_string(),
                    serde_json::Value::String("file".to_string()),
                );
            }

        #[derive(Deserialize)]
        #[serde(tag = "type")]
        enum TaggedSound {
            #[serde(rename = "event", alias = "minecraft:event")]
            SoundEvent {
                name: crate::resource_pack::identifier::Identifier,
                #[serde(default)]
                volume: Option<f32>,
                #[serde(default)]
                pitch: Option<f32>,
                #[serde(default, deserialize_with = "deserialize_optional_i32")]
                weight: Option<i32>,
                #[serde(default)]
                preload: Option<bool>,
                #[serde(default)]
                stream: Option<bool>,
                #[serde(default, deserialize_with = "deserialize_optional_i32")]
                attenuation_distance: Option<i32>,
            },
            #[serde(rename = "file", alias = "minecraft:file")]
            File {
                name: crate::resource_pack::identifier::SoundId,
                #[serde(default)]
                volume: Option<f32>,
                #[serde(default)]
                pitch: Option<f32>,
                #[serde(default, deserialize_with = "deserialize_optional_i32")]
                weight: Option<i32>,
                #[serde(default)]
                preload: Option<bool>,
                #[serde(default)]
                stream: Option<bool>,
                #[serde(default, deserialize_with = "deserialize_optional_i32")]
                attenuation_distance: Option<i32>,
            },
        }

        let tagged = serde_json::from_value::<TaggedSound>(value)
            .map_err(D::Error::custom)?;

        Ok(match tagged {
            TaggedSound::SoundEvent {
                name, volume, pitch, weight, preload, stream,
                attenuation_distance,
            } => Self::SoundEvent {
                name, volume, pitch, weight, preload, stream,
                attenuation_distance, r#type: "event".to_string(),
            },
            TaggedSound::File {
                name, volume, pitch, weight, preload, stream,
                attenuation_distance,
            } => Self::File {
                name, volume, pitch, weight, preload, stream,
                attenuation_distance,
            },
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_sounds_SoundEventRegistration {
    /// The sound files this sound event uses. One sound is randomly selected to play when the event is triggered. Defaults to assumed path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sounds: Option<Vec<JavaAssetsSoundsSoundEventRegistrationSounds>>,
    /// If true the sounds listed should replace the ones listed in the minecraft sounds.json for this sound event.
    /// False if the sounds listed should be added. If undefined. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<bool>,
    /// Translated as the subtitle when Show Subtitles is enabled. Section sign formatting codes are supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
}

pub type java_assets_sounds_Sounds = HashMap<String, java_assets_sounds_SoundEventRegistration>;
