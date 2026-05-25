use crate::minecraft::builtin_files;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
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

pub static GLOBAL_ID_USAGE_COUNTER: OnceLock<IdUsageCounter> = OnceLock::new();

#[derive(Debug, Default)]
pub struct IdUsageCounter {
    pub model_counter: DashMap<String, AtomicUsize>,
    pub texture_counter: DashMap<String, AtomicUsize>,
    pub sound_counter: DashMap<String, AtomicUsize>,
}

impl IdUsageCounter {
    pub fn increment_counter(&self, id: String, category: IdCategory) {
        match category {
            IdCategory::Model => {
                if builtin_files::is_in_models(id.as_str()) {
                    self.model_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            IdCategory::Texture => {
                if !builtin_files::is_in_textures(id.as_str()) {
                    self.texture_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            IdCategory::Sound => {
                if builtin_files::is_in_sounds(id.as_str()) {
                    self.sound_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

pub fn set_id_usage_counter(c: IdUsageCounter) {
    GLOBAL_ID_USAGE_COUNTER.set(c).ok();
}
