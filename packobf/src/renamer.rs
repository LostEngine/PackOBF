use crate::minecraft::builtin_files;
use crate::minecraft::builtin_files::AtlasType;
use crate::resource_pack::files::atlas::{Atlas, Source};
use crate::resource_pack::files::font::FontProvider;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::{self, Mapping};
use crate::resource_pack::pack::FrozenResourcePack;
use crate::{profile_scope, LogLevel, LogMessage};
use std::collections::HashMap;
use std::mem::take;
use tokio::sync::mpsc::UnboundedSender;

pub fn rename_files(
    logger: &UnboundedSender<LogMessage>,
    pack: &mut FrozenResourcePack,
    mapping: &mut Mapping,
) {
    profile_scope!(std::any::type_name_of_val(&rename_files));
    let id_counter = &mapping::get_id_usage_counter();
    rename_overlays(pack, &mut mapping.overlay_mappings);
    rename_models(pack, &mut mapping.model_mappings, id_counter);
    rename_textures(logger, pack, &mut mapping.texture_mappings, id_counter);
    rename_sounds(pack, &mut mapping.sound_mappings, id_counter);
}

fn rename_overlays(pack: &mut FrozenResourcePack, mapping: &mut HashMap<String, String>) {
    profile_scope!(std::any::type_name_of_val(&rename_overlays));
    if let Some(mut mcmeta) = pack.pack_mcmeta.take() {

        if let Some(overlay) = mcmeta.overlays.as_mut() {
            if let Some(entries) = overlay.entries.as_mut() {
                for (count, entry) in entries.iter_mut().enumerate() {
                    let new_name = generate_short_name(count);
                    mapping.insert(entry.directory.clone(), new_name.clone());
                    entry.directory = new_name;
                }
            }
        }
        pack.pack_mcmeta = Some(mcmeta);
    }
}

fn rename_sounds(
    pack: &mut FrozenResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_sounds));
    let mut sounds = take(&mut pack.sounds);

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

    let mut updated_sounds = Vec::with_capacity(sounds.len());
    for (count, (_key, mut sound)) in sounds.into_iter().enumerate() {
        let identifier = sound.identifier.to_string();
        if let Some(mapped) = mapping.get(&identifier) {
            sound.identifier.path = mapped.clone();
            let new_path = sound.path();
            updated_sounds.push((new_path, sound));
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping.insert(identifier, new_identifier.to_string());
            sound.identifier = new_identifier;
            let new_path = sound.path();
            updated_sounds.push((new_path, sound));
        }
    }

    updated_sounds.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    pack.sounds = updated_sounds;
}

fn rename_textures(
    logger: &UnboundedSender<LogMessage>,
    pack: &mut FrozenResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_textures));
    let mut textures = take(&mut pack.textures);

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

    let mut updated_textures = Vec::with_capacity(textures.len());
    for (key, mut texture) in textures {
        let identifier = texture.identifier.to_string();

        if let Some(mapped) = mapping.get(&identifier) {
            texture.identifier.path = mapped.clone();
            let new_path = texture.path();

            let old_mcmeta_key = format!("{key}.mcmeta");
            if let Some(idx) = pack.json_files.iter().position(|(k, _)| k == &old_mcmeta_key) {
                let (_, mcmeta) = pack.json_files.remove(idx);
                pack.json_files.push((format!("{new_path}.mcmeta"), mcmeta));
            }

            updated_textures.push((new_path, texture));
        } else {
            let mut in_items = false;
            let mut in_blocks = false;
            let in_font = font_textures.contains(&texture.identifier.to_string());
            let mut aliases = Vec::new();

            for (_, atlas) in &pack.atlases {
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
                // If skipping, retain the item unchanged
                updated_textures.push((key, texture));
                continue;
            };
            let count = per_folder_count.entry(prefix.to_string()).or_insert(0);
            let path = prefix.to_owned() + generate_short_name(*count).as_str();
            let new_identifier = Identifier::new("_", path);
            mapping.insert(identifier, new_identifier.to_string());
            texture.identifier = new_identifier;
            let new_path = texture.path();

            // Handle .mcmeta relocation in vector-based json_files
            let old_mcmeta_key = format!("{key}.mcmeta");
            if let Some(idx) = pack.json_files.iter().position(|(k, _)| k == &old_mcmeta_key) {
                let (_, mcmeta) = pack.json_files.remove(idx);
                pack.json_files.push((format!("{new_path}.mcmeta"), mcmeta));
            }

            updated_textures.push((new_path, texture));
            *count += 1;
        }
    }

    updated_textures.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    pack.textures = updated_textures;

    pack.json_files.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    rebuild_atlas(pack);
}

fn get_font_textures(pack: &FrozenResourcePack) -> Vec<String> {
    let mut font_textures = Vec::new();
    for (_, font) in pack.fonts.iter() {
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

fn rebuild_atlas(pack: &mut FrozenResourcePack) {
    let mut item_atlas_exists = false;
    let mut block_atlas_exists = false;
    for (_, atlas) in pack.atlases.iter_mut() {
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
        pack.atlases.push((atlas.path(), atlas));
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
        pack.atlases.push((atlas.path(), atlas));
    }
}

fn rename_models(
    pack: &mut FrozenResourcePack,
    mapping: &mut HashMap<String, String>,
    id_counter: &mapping::IdUsageCounter,
) {
    profile_scope!(std::any::type_name_of_val(&rename_models));
    let mut models = take(&mut pack.models);

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

    let mut updated_models = Vec::with_capacity(models.len());

    for (count, (_key, mut model)) in models.into_iter().enumerate() {
        let identifier = model.identifier.to_string();
        if let Some(mapped) = mapping.get(&identifier) {
            model.identifier.path = mapped.clone();
            let new_path = model.path();
            updated_models.push((new_path, model));
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping.insert(identifier, new_identifier.to_string());
            model.identifier = new_identifier;
            let new_path = model.path();
            updated_models.push((new_path, model));
        }
    }

    updated_models.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    pack.models = updated_models;
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
