use crate::minecraft::builtin_files;
use crate::resource_pack::identifier::Identifier;
use arc_swap::ArcSwap;
use crossbeam::queue::SegQueue;
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
    pub texture_mappings: TextureMappings,
    pub sound_mappings: HashMap<String, String>,
    pub overlay_mappings: HashMap<String, String>,
    pub font_textures: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TextureMappings {
    /// Mappings for where the textures will be relocated to
    pub assets_mapping: HashMap<Identifier, Identifier>,
    /// Mappings for references to textures
    pub refenrence_mappings: HashMap<String, String>,
}

impl TextureMappings {
    pub(crate) fn asset_identifier<'a>(&'a self, identifier: &'a Identifier) -> &'a Identifier {
        self.assets_mapping.get(identifier).unwrap_or(identifier)
    }
}

impl Mapping {
    pub fn apply_mapping(&self, id: &str, category: IdCategory) -> String {
        match category {
            IdCategory::Model => {
                if let Some(mapped) = self.model_mappings.get(id) {
                    return mapped.clone();
                }
            }
            IdCategory::Texture => {
                if let Some(mapped) = self.texture_mappings.refenrence_mappings.get(id) {
                    return mapped.clone();
                }
            }
            IdCategory::Sound => {
                if let Some(mapped) = self.sound_mappings.get(id) {
                    return mapped.clone();
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
    pub model_counter: DashMap<Identifier, AtomicUsize>,
    pub texture_counter: DashMap<Identifier, AtomicUsize>,
    pub sound_counter: DashMap<Identifier, AtomicUsize>,
    pub font_textures: SegQueue<String>,
}

impl IdUsageCounter {
    pub fn increment_counter(&self, id: Identifier, category: IdCategory) {
        match category {
            IdCategory::Model => {
                if !builtin_files::is_in_models(&id.to_string()) {
                    self.model_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            IdCategory::Texture => {
                if !builtin_files::is_in_textures(&id.to_string()) {
                    self.texture_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            IdCategory::Sound => {
                if !builtin_files::is_in_sounds(&id.to_string()) {
                    self.sound_counter
                        .entry(id)
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn get_usage_count(&self, id: &Identifier, category: IdCategory) -> usize {
        match category {
            IdCategory::Model => self
                .model_counter
                .get(id)
                .map_or(0, |counter| counter.load(Ordering::Relaxed)),
            IdCategory::Texture => self
                .texture_counter
                .get(id)
                .map_or(0, |counter| counter.load(Ordering::Relaxed)),
            IdCategory::Sound => self
                .sound_counter
                .get(id)
                .map_or(0, |counter| counter.load(Ordering::Relaxed)),
        }
    }
}

pub fn set_id_usage_counter(c: IdUsageCounter) {
    GLOBAL_ID_USAGE_COUNTER.store(Arc::new(c));
}

pub fn get_id_usage_counter() -> Arc<IdUsageCounter> {
    GLOBAL_ID_USAGE_COUNTER.load_full()
}
