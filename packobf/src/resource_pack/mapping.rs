use std::collections::HashMap;
use std::sync::OnceLock;

pub enum IdCategory {
    Model,
    Texture,
    Sound,
}

pub static GLOBAL_MAPPING: OnceLock<Mapping> = OnceLock::new();

#[derive(Clone, Debug, Default)]
pub struct Mapping {
    pub model_mappings: HashMap<String, String>,
    pub texture_mappings: HashMap<String, String>,
    pub sound_mappings: HashMap<String, String>,
    pub overlay_mappings: HashMap<String, String>,
}

impl Mapping {
    pub fn apply_mapping(&self, id: &str, category: IdCategory) -> String {
        match category {
            IdCategory::Model => {
                if let Some(mapped) = &self.model_mappings.get(id) {
                    return mapped.to_string();
                }
            }
            IdCategory::Texture => {
                if let Some(mapped) = &self.texture_mappings.get(id) {
                    return mapped.to_string();
                }
            }
            IdCategory::Sound => {
                if let Some(mapped) = &self.sound_mappings.get(id) {
                    return mapped.to_string();
                }
            }
        }
        id.to_string()
    }
}

pub fn set_mappings(m: Mapping) {
    GLOBAL_MAPPING.set(m).ok();
}
