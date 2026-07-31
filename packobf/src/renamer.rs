use crate::minecraft::builtin_files;
use crate::minecraft::builtin_files::AtlasType;
use crate::resource_pack::files::atlas::{Atlas, Source};
use crate::resource_pack::files::font::FontProvider;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::texture::Texture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::{self, Mapping};
use crate::resource_pack::pack::ResourcePack;
use crate::{profile_scope, LogLevel, LogMessage};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

pub fn rename_files(
    logger: &UnboundedSender<LogMessage>,
    pack: &ResourcePack,
    mapping: &mut Mapping,
) {
    profile_scope!(std::any::type_name_of_val(&rename_files));
    let id_counter = &mapping::get_id_usage_counter();
    rayon::scope(|s| {
        s.spawn(|_| rename_overlays(pack, &mut mapping.overlay_mappings));
        s.spawn(|_| rename_models(pack, &mut mapping.model_mappings, id_counter));
        s.spawn(|_| rename_textures(logger, &pack, &mut mapping.texture_mappings, id_counter));
        s.spawn(|_| rename_sounds(pack, &mut mapping.sound_mappings, id_counter));
    });
}

fn rename_overlays(pack: &ResourcePack, mapping: &mut HashMap<String, String>) {
    profile_scope!(std::any::type_name_of_val(&rename_overlays));
    let mut count = 0;
    if let Some(mut mcmeta) = pack.json_files.get_mut("pack.mcmeta") {
        if let Some(entries) = mcmeta
            .value_mut()
            .content
            .pointer_mut("/overlays/entries")
            .and_then(|v| v.as_array_mut())
        {
            for entry in entries {
                if let Some(dir) = entry.get_mut("directory") {
                    let current_name = dir.as_str().map(|s| s.to_string());
                    if let Some(current_name) = current_name {
                        let new_name = generate_short_name(count);
                        *dir = json!(new_name.clone());
                        mapping.insert(current_name, new_name);
                        count += 1;
                    }
                }
            }
        }
    }
}

fn rename_sounds(
    pack: &ResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_sounds));
    let mut sounds: Vec<(String, Sound)> = pack
        .sounds
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    sounds.retain(|(_, x)| {
        !(x.identifier.namespace == "minecraft"
            && builtin_files::is_in_sounds(x.identifier.to_string().as_str()))
    });

    sounds.sort_by(|(_, a), (_, b)| {
        let a_id = a.identifier.to_string();
        let b_id = b.identifier.to_string();

        let a_usage = id_counter.get_usage_count(&a_id, mapping::IdCategory::Sound);
        let b_usage = id_counter.get_usage_count(&b_id, mapping::IdCategory::Sound);

        // Higher usage first
        b_usage
            .cmp(&a_usage)
            // Stable deterministic fallback so the same resource pack is generated each time
            .then_with(|| a_id.cmp(&b_id))
    });

    for (count, (key, mut sound)) in sounds.into_iter().enumerate() {
        let identifier = sound.identifier.to_string();
        if let Some(mapped) = mapping.get(&identifier) {
            sound.identifier.path = mapped.clone();
            pack.sounds.remove(&key);
            pack.sounds.insert(sound.path(), sound);
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping.insert(identifier, new_identifier.to_string());
            sound.identifier = new_identifier;
            pack.sounds.remove(&key);
            pack.sounds.insert(sound.path(), sound);
        }
    }
}

fn rename_textures(
    logger: &UnboundedSender<LogMessage>,
    pack: &&ResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_textures));
    let mut textures: Vec<(String, Texture)> = pack
        .textures
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    textures.retain(|(_, x)| {
        !(x.identifier.namespace == "minecraft"
            && builtin_files::is_in_textures(x.identifier.to_string().as_str()))
    });

    textures.sort_by(|(_, a), (_, b)| {
        let a_id = a.identifier.to_string();
        let b_id = b.identifier.to_string();

        let a_usage = id_counter.get_usage_count(&a_id, mapping::IdCategory::Texture);
        let b_usage = id_counter.get_usage_count(&b_id, mapping::IdCategory::Texture);

        // Higher usage first
        b_usage
            .cmp(&a_usage)
            // Stable deterministic fallback so the same resource pack is generated each time
            .then_with(|| a_id.cmp(&b_id))
    });

    let mut per_folder_count: HashMap<String, usize> = HashMap::new();
    let font_textures = get_font_textures(pack);
    for (key, mut texture) in textures.into_iter() {
        let identifier = texture.identifier.to_string();
        if let Some(mapped) = mapping.get(&identifier) {
            texture.identifier.path = mapped.clone();
            pack.textures.remove(&key);
            let new_path = texture.path();
            if let Some(mcmeta) = pack.json_files.remove(format!("{}.mcmeta", key).as_str()) {
                pack.json_files
                    .insert(format!("{}.mcmeta", new_path), mcmeta.1);
            };
            pack.textures.insert(new_path, texture);
        } else {
            let mut in_items = false;
            let mut in_blocks = false;
            let in_font = font_textures.contains(&texture.identifier.to_string());
            let mut aliases = Vec::new();
            for atlas in &pack.atlases {
                if atlas.overlay != texture.overlay {
                    continue;
                }
                match atlas.atlas_type {
                    AtlasType::Blocks => {
                        if let Some(id) = atlas.get_identifier(&texture.identifier) {
                            in_blocks = true;
                            aliases.push(id);
                        }
                    }
                    AtlasType::Items => {
                        if let Some(id) = atlas.get_identifier(&texture.identifier) {
                            in_items = true;
                            aliases.push(id);
                        }
                    }
                    _ => {}
                }
            }
            match builtin_files::get_atlas(texture.identifier.path.as_str()) {
                Some(AtlasType::Blocks) => {
                    in_blocks = true;
                }
                Some(AtlasType::Items) => {
                    in_items = true;
                }
                _ => {}
            }
            let prefix = if in_blocks {
                if in_items {
                    let _ = logger.send(LogMessage {
                        level: LogLevel::Warning,
                        message: format!(
                            "'{}' is both in blocks and items atlas. Using blocks atlas.",
                            texture.path()
                        ),
                    });
                }
                "b/"
            } else if in_items {
                "i/"
            } else if in_font {
                ""
            } else {
                continue;
            };
            let count = per_folder_count.entry(prefix.to_string()).or_insert(0);
            let path = prefix.to_owned() + generate_short_name(*count).as_str();
            let new_identifier = Identifier::new("_", path);
            mapping.insert(identifier, new_identifier.to_string());
            texture.identifier = new_identifier;
            pack.textures.remove(&key);
            let new_path = texture.path();
            if let Some(mcmeta) = pack.json_files.remove(format!("{}.mcmeta", key).as_str()) {
                pack.json_files
                    .insert(format!("{}.mcmeta", new_path), mcmeta.1);
            };
            pack.textures.insert(new_path, texture);
            *count += 1;
        }
    }
    rebuild_atlas(pack);
}

fn get_font_textures(pack: &ResourcePack) -> Vec<String> {
    let mut font_textures = Vec::new();
    for font in pack.fonts.iter() {
        for provider in font.providers.iter() {
            if let FontProvider::Bitmap { file, .. } = provider {
                let id = if file.0.namespace == "minecraft" {
                    file.0.path.to_string()
                } else {
                    format!("{}:{}", file.0.namespace, file.0.path)
                };
                font_textures.push(id);
            }
        }
    }
    font_textures
}

fn rebuild_atlas(pack: &ResourcePack) {
    let mut item_atlas_exists = false;
    let mut block_atlas_exists = false;
    for mut atlas in pack.atlases.iter_mut() {
        atlas.sources.retain(|source| !matches!(source, Source::Directory { .. } | Source::Single { .. }));
        match atlas.atlas_type {
            AtlasType::Blocks => {
                if atlas.overlay.is_empty() {
                    block_atlas_exists = true;
                }
                atlas.sources.push(Source::Directory {
                    source: "b".to_string(),
                    prefix: "b/".to_string(),
                });
            }
            AtlasType::Items => {
                if atlas.overlay.is_empty() {
                    item_atlas_exists = true;
                }
                atlas.sources.push(Source::Directory {
                    source: "i".to_string(),
                    prefix: "i/".to_string(),
                });
            }
            _ => {}
        }
    }
    if !block_atlas_exists {
        let atlas = Atlas {
            overlay: "".to_string(),
            sources: vec![Source::Directory {
                source: "b".to_string(),
                prefix: "b/".to_string(),
            }],
            atlas_type: AtlasType::Blocks,
        };
        pack.atlases.insert(atlas.path(), atlas);
    }
    if !item_atlas_exists {
        let atlas = Atlas {
            overlay: "".to_string(),
            sources: vec![Source::Directory {
                source: "i".to_string(),
                prefix: "i/".to_string(),
            }],
            atlas_type: AtlasType::Items,
        };
        pack.atlases.insert(atlas.path(), atlas);
    }
}

fn rename_models(
    pack: &ResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_models));
    let mut models: Vec<(String, Model)> = pack
        .models
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    models.retain(|(_, x)| {
        !(x.identifier.namespace == "minecraft"
            && builtin_files::is_in_models(x.identifier.to_string().as_str()))
    });

    models.sort_by(|(_, a), (_, b)| {
        let a_id = a.identifier.to_string();
        let b_id = b.identifier.to_string();

        let a_usage = id_counter.get_usage_count(&a_id, mapping::IdCategory::Model);
        let b_usage = id_counter.get_usage_count(&b_id, mapping::IdCategory::Model);

        // Higher usage first
        b_usage
            .cmp(&a_usage)
            // Stable deterministic fallback so the same resource pack is generated each time
            .then_with(|| a_id.cmp(&b_id))
    });

    for (count, (key, mut model)) in models.into_iter().enumerate() {
        let identifier = model.identifier.to_string();
        if let Some(mapped) = mapping.get(&identifier) {
            model.identifier.path = mapped.clone();
            pack.models.remove(&key);
            pack.models.insert(model.path(), model);
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping.insert(identifier, new_identifier.to_string());
            model.identifier = new_identifier;
            pack.models.remove(&key);
            pack.models.insert(model.path(), model);
        }
    }
}

fn generate_short_name(mut id: usize) -> String {
    let charset = "abcdefghijklmnopqrstuvwxyz0123456789_-";
    let base = charset.len();
    let bytes = charset.as_bytes();

    let mut name = String::new();
    loop {
        let rem = id % base;
        name.push(bytes[rem] as char);
        id /= base;

        if id == 0 {
            break;
        }
    }
    name
}
