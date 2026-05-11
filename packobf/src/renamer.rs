use crate::minecraft::builtin_files;
use crate::minecraft::builtin_files::AtlasType;
use crate::resource_pack::files::atlas::Source;
use crate::resource_pack::files::font::FontProvider;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::texture::Texture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::Mapping;
use crate::resource_pack::resource_pack::ResourcePack;
use crate::{LogLevel, LogMessage};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

pub fn rename_files(
    logger: &UnboundedSender<LogMessage>,
    pack: &ResourcePack,
    mapping: &mut Mapping,
) {
    rename_overlays(pack, mapping);
    rename_models(pack, mapping);
    rename_textures(logger, &pack, mapping);
    rename_sounds(pack, mapping);
}

fn rename_overlays(pack: &ResourcePack, mapping: &mut Mapping) {
    let mut count = 0;
    match pack.json_files.get_mut("pack.mcmeta") {
        Some(mut mcmeta) => {
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
                            mapping.overlay_mappings.insert(current_name, new_name);
                            count += 1;
                        }
                    }
                }
            }
        }
        None => {}
    }
}

fn rename_sounds(pack: &ResourcePack, mapping: &mut Mapping) {
    let mut count = 0;
    for x in pack.sounds.clone().iter() {
        let identifier = x.identifier.to_string();
        if x.identifier.namespace == "minecraft" {
            // Skip sounds if they are overwriting Minecraft files
            if builtin_files::is_in_sounds(identifier.as_str()) {
                continue;
            }
        }
        if let Some(mapped) = mapping.sound_mappings.get(&identifier) {
            let mut new_sound: Sound = x.clone();
            new_sound.identifier.path = mapped.clone();
            pack.sounds.remove(&x.key().to_string());
            pack.sounds.insert(new_sound.path(), new_sound);
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping
                .sound_mappings
                .insert(identifier, new_identifier.to_string());
            let mut new_sound: Sound = x.clone();
            new_sound.identifier = new_identifier;
            pack.sounds.remove(&x.key().to_string());
            pack.sounds.insert(new_sound.path(), new_sound);
            count += 1;
        }
    }
}

fn rename_textures(
    logger: &UnboundedSender<LogMessage>,
    pack: &&ResourcePack,
    mapping: &mut Mapping,
) {
    let mut per_folder_count: HashMap<String, usize> = HashMap::new();
    let font_textures = get_font_textures(pack);
    for x in pack.textures.clone().iter() {
        let identifier = x.identifier.to_string();
        if x.identifier.namespace == "minecraft" {
            // Skip textures if they are overwriting Minecraft files
            if builtin_files::is_in_textures(identifier.as_str()) {
                continue;
            }
        }
        if let Some(mapped) = mapping.texture_mappings.get(&identifier) {
            let mut new_texture: Texture = x.clone();
            new_texture.identifier.path = mapped.clone();
            pack.textures.remove(x.key());
            let new_path = new_texture.path();
            if let Some(mcmeta) = pack
                .json_files
                .remove(format!("{}.mcmeta", x.key()).as_str())
            {
                pack.json_files
                    .insert(format!("{}.mcmeta", new_path), mcmeta.1);
            };
            pack.textures.insert(new_path, new_texture);
        } else {
            let mut in_items = false;
            let mut in_blocks = false;
            let in_font = font_textures.contains(&x.identifier.to_string());
            let mut aliases = Vec::new();
            for atlas in &pack.atlases {
                if atlas.overlay != x.overlay {
                    continue;
                }
                match atlas.atlas_type {
                    AtlasType::Blocks => {
                        if let Some(id) = atlas.get_identifier(&x.identifier) {
                            in_blocks = true;
                            aliases.push(id);
                        }
                    }
                    AtlasType::Items => {
                        if let Some(id) = atlas.get_identifier(&x.identifier) {
                            in_items = true;
                            aliases.push(id);
                        }
                    }
                    _ => {}
                }
            }
            match builtin_files::get_atlas(x.identifier.path.as_str()) {
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
                            x.path()
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
            mapping
                .texture_mappings
                .insert(identifier, new_identifier.to_string());
            let mut new_texture: Texture = x.clone();
            new_texture.identifier = new_identifier;
            pack.textures.remove(x.key());
            let new_path = new_texture.path();
            if let Some(mcmeta) = pack
                .json_files
                .remove(format!("{}.mcmeta", x.key()).as_str())
            {
                pack.json_files
                    .insert(format!("{}.mcmeta", new_path), mcmeta.1);
            };
            pack.textures.insert(new_path, new_texture);
            *count += 1;
        }
    }
    rebuild_atlas(pack);
}

fn get_font_textures(pack: &ResourcePack) -> Vec<String> {
    let mut font_textures = Vec::new();
    for font in pack.fonts.iter() {
        for provider in font.providers.iter() {
            match provider {
                FontProvider::Bitmap { file, .. } => {
                    let id = if file.0.namespace == "minecraft" {
                        format!("{}", file.0.path)
                    } else {
                        format!("{}:{}", file.0.namespace, file.0.path)
                    };
                    font_textures.push(id);
                }
                _ => {}
            }
        }
    }
    font_textures
}

fn rebuild_atlas(pack: &ResourcePack) {
    for mut atlas in pack.atlases.iter_mut() {
        atlas.sources.retain(|source| match source {
            Source::Directory { .. } => false,
            Source::Single { .. } => false,
            _ => true,
        });
        match atlas.atlas_type {
            AtlasType::Blocks => {
                atlas.sources.push(Source::Directory {
                    source: "b".to_string(),
                    prefix: "b/".to_string(),
                });
            }
            AtlasType::Items => {
                atlas.sources.push(Source::Directory {
                    source: "i".to_string(),
                    prefix: "i/".to_string(),
                });
            }
            _ => {}
        }
    }
}

fn rename_models(pack: &ResourcePack, mapping: &mut Mapping) {
    let mut count = 0;
    for x in pack.models.clone().iter() {
        let identifier = x.identifier.to_string();
        if x.identifier.namespace == "minecraft" {
            // Skip models if they are overwriting Minecraft files
            if builtin_files::is_in_models(identifier.as_str()) {
                continue;
            }
        }
        if let Some(mapped) = mapping.model_mappings.get(&identifier) {
            let mut new_model: Model = x.clone();
            new_model.identifier.path = mapped.clone();
            pack.models.remove(&x.key().to_string());
            pack.models.insert(new_model.path(), new_model);
        } else {
            let new_identifier = Identifier::new("_", generate_short_name(count));
            mapping
                .model_mappings
                .insert(identifier, new_identifier.to_string());
            let mut new_model: Model = x.clone();
            new_model.identifier = new_identifier;
            pack.models.remove(&x.key().to_string());
            pack.models.insert(new_model.path(), new_model);
            count += 1;
        }
    }
}

fn generate_short_name(mut id: usize) -> String {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-"; // We do not care about convention
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
