use crate::minecraft::builtin_files;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Clone, Copy)]
pub enum IdCategory {
    Model,
    Texture,
    Sound,
}

pub static GLOBAL_MAPPING: LazyLock<ArcSwap<Mapping>> =
    LazyLock::new(|| ArcSwap::from_pointee(Mapping::default()));

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
    GLOBAL_MAPPING.store(Arc::new(m));
}

pub fn get_mappings() -> Arc<Mapping> {
    GLOBAL_MAPPING.load_full()
}

pub static GLOBAL_ID_USAGE_COUNTER: LazyLock<ArcSwap<IdUsageCounter>> =
    LazyLock::new(|| ArcSwap::from_pointee(IdUsageCounter::default()));

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

    pub fn get_usage_count(&self, id: &str, category: IdCategory) -> usize {
        match category {
            IdCategory::Model => self
                .model_counter
                .get(id)
                .map(|counter| counter.load(Ordering::Relaxed))
                .unwrap_or(0),
            IdCategory::Texture => self
                .texture_counter
                .get(id)
                .map(|counter| counter.load(Ordering::Relaxed))
                .unwrap_or(0),
            IdCategory::Sound => self
                .sound_counter
                .get(id)
                .map(|counter| counter.load(Ordering::Relaxed))
                .unwrap_or(0),
        }
    }
}

pub fn set_id_usage_counter(c: IdUsageCounter) {
    GLOBAL_ID_USAGE_COUNTER.store(Arc::new(c));
}

pub fn get_id_usage_counter() -> Arc<IdUsageCounter> {
    GLOBAL_ID_USAGE_COUNTER.load_full()
}
