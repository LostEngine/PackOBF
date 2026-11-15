use crate::minecraft::builtin_files;
use crate::minecraft::builtin_files::AtlasType;
use crate::minecraft::json::atlas::{java_assets_atlas_Atlas, java_assets_atlas_SpriteSource};
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping::{self, Mapping, TextureMappings};
use crate::resource_pack::pack::{Asset, ResourcePack};
use crate::{profile_scope, LogLevel, LogMessage};
use itertools::Itertools;
use rayon::ThreadPool;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;


// Source is the folder where the assets are located, and Prefix is the prefix that will be added to
// the identifier. Source and Prefix are intentionally different to confuse people trying to extract
// resources from the pack.
const BLOCKS_SOURCE: &str = ".";
const BLOCKS_SOURCE_PATH: &str = "./";
const BLOCKS_PREFIX: &str = "_/";

const ITEMS_SOURCE: &str = "_";
const ITEMS_SOURCE_PATH: &str = "_/";
const ITEMS_PREFIX: &str = "./";

macro_rules! generate_mappings {
    ($asset:path, $category:expr, $pack:expr, $id_counter:expr, $mapping:expr, $is_in_vanilla:expr) => {
        let mut ids: Vec<&Identifier> = $pack.assets
            .iter()
            .filter_map(|asset| {
                if let $asset(asset) = asset {
                    let id = &asset.identifier;
                    if id.namespace != "minecraft" || !$is_in_vanilla(&id.to_string()) {
                        Some(id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        ids.sort_by(|a, b| {
            let a_usage = $id_counter.get_usage_count(&a, $category);
            let b_usage = $id_counter.get_usage_count(&b, $category);

            // Higher usage first
            b_usage
                .cmp(&a_usage)
                // Stable deterministic fallback so the same resource pack is generated each time
                .then_with(|| a.to_string().cmp(&b.to_string()))
        });
        let mut count = 0;
        for identifier in ids {
            let id_string = identifier.to_string();
            if !$mapping.contains_key(&id_string) {
                let new_identifier = format!("_:{}", generate_short_name(count, true));
                $mapping.insert(id_string, new_identifier);
                count += 1;
            }
        }
    };
}

pub fn rename_files(
    logger: &UnboundedSender<LogMessage>,
    pack: &mut ResourcePack,
    mapping: &mut Mapping,
    thread_pool: &ThreadPool,
) {
    profile_scope!(std::any::type_name_of_val(&rename_files));
    let id_counter = &mapping::get_id_usage_counter();
    rename_overlays(pack, &mut mapping.overlay_mappings);

    thread_pool.install(|| {
        rayon::scope(|s| {
            s.spawn(|_| {
                profile_scope!("generate_sound_mappings");
                generate_mappings!(Asset::Sound, mapping::IdCategory::Sound, &pack, id_counter, &mut mapping.sound_mappings, builtin_files::is_in_sounds);
            });
            s.spawn(|_| {
                profile_scope!("generate_model_mappings");
                generate_mappings!(Asset::Model, mapping::IdCategory::Model, &pack, id_counter, &mut mapping.model_mappings, builtin_files::is_in_models);
            });
            generate_texture_mappings(logger, pack, id_counter, &mut mapping.texture_mappings);
        });
    });
    rebuild_atlases(pack, &mapping.texture_mappings);
}

fn rename_overlays(pack: &mut ResourcePack, mapping: &mut HashMap<String, String>) {
    profile_scope!(std::any::type_name_of_val(&rename_overlays));
    if let Some(mcmeta) = pack.pack_mcmeta.as_mut() &&
        let Some(overlay) = mcmeta.overlays.as_mut() &&
        let Some(entries) = overlay.entries.as_mut() {
        for (count, entry) in entries.iter_mut().enumerate() {
            let new_name = generate_short_name(count, false);
            mapping.insert(entry.directory.clone(), new_name.clone());
            entry.directory = new_name;
        }
    }
}

fn generate_texture_mappings(
    logger: &UnboundedSender<LogMessage>,
    pack: &ResourcePack,
    id_counter: &mapping::IdUsageCounter,
    mappings: &mut TextureMappings,
) {
    profile_scope!(std::any::type_name_of_val(&generate_texture_mappings));
    let mut textures: Vec<&Identifier> = pack.assets
        .iter()
        .filter_map(|asset| {
            if let Asset::AssetTexture(texture) = asset {
                let id = &texture.identifier;
                if id.namespace != "minecraft" ||
                    !builtin_files::is_in_textures(id.to_string().as_str()) {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    textures.sort_by(|a, b| {
        let a_usage = texture_usage(pack, a, id_counter);
        let b_usage = texture_usage(pack, b, id_counter);

        // Higher usage first
        b_usage
            .cmp(&a_usage)
            // Stable deterministic fallback so the same resource pack is generated each time
            .then_with(|| a.to_string().cmp(&b.to_string()))
    });

    let mut per_folder_count: HashMap<String, usize> = HashMap::new();
    let font_textures: Vec<String> = std::iter::from_fn(|| id_counter.font_textures.pop()).collect();

    for texture in textures {
        let texture_id = texture.to_string();
        let mut in_items = false;
        let mut in_blocks = false;
        let in_font = font_textures.contains(&texture_id);
        let mut aliases = Vec::new();

        for asset in &pack.assets {
            if let Asset::Atlas(asset) = asset {
                let sprites = asset.get_sprites(texture);
                if sprites.is_empty() {
                    continue;
                }
                match asset.atlas_type {
                    AtlasType::Blocks => in_blocks = true,
                    AtlasType::Items => in_items = true,
                    _ => continue,
                }
                aliases.extend(sprites);
            }
        }
        match builtin_files::get_atlas(texture.path.as_str()) {
            Some(AtlasType::Blocks) => {
                in_blocks = true;
            }
            Some(AtlasType::Items) => {
                in_items = true;
            }
            _ => {}
        }
        let (asset_prefix, reference_prefix) = if in_blocks {
            if in_items {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Warning,
                    message: format!(
                        "'{}' is both in blocks and items atlas. Using blocks atlas.",
                        texture_id
                    ),
                });
            }
            (BLOCKS_SOURCE_PATH, BLOCKS_PREFIX)
        } else if in_items {
            (ITEMS_SOURCE_PATH, ITEMS_PREFIX)
        } else if in_font {
            ("", "")
        } else {
            continue;
        };
        let count = per_folder_count.entry(asset_prefix.to_string()).or_insert(0);

        let name = generate_short_name(*count, true);
        mappings.assets_mapping.insert(texture.clone(), Identifier::new("_", format!("{asset_prefix}{name}")));

        mappings.refenrence_mappings.insert(texture_id, format!("_:{reference_prefix}{name}"));
        for alias in aliases {
            mappings.refenrence_mappings.insert(alias.to_string(), format!("_:{reference_prefix}{name}"));
        }
        *count += 1;
    }
}

fn texture_usage(
    pack: &ResourcePack,
    id: &Identifier,
    counter: &mapping::IdUsageCounter,
) -> usize {
    profile_scope!(std::any::type_name_of_val(&texture_usage));
    let direct = counter.get_usage_count(id, mapping::IdCategory::Texture);
    pack.assets
        .iter()
        .filter_map(|asset| {
            if let Asset::Atlas(atlas) = asset {
                Some(atlas)
            } else {
                None
            }
        })
        .flat_map(|atlas| atlas.get_sprites(id))
        .unique()
        .map(|sprite| counter.get_usage_count(&sprite, mapping::IdCategory::Texture))
        .fold(direct, usize::saturating_add)
}

fn rebuild_atlases(pack: &mut ResourcePack, mappings: &TextureMappings) {
    profile_scope!(std::any::type_name_of_val(&rebuild_atlases));
    for (atlas_type, source, asset_prefix, sprite_prefix) in [
        (AtlasType::Blocks, BLOCKS_SOURCE, BLOCKS_SOURCE_PATH, BLOCKS_PREFIX),
        (AtlasType::Items, ITEMS_SOURCE, ITEMS_SOURCE_PATH, ITEMS_PREFIX),
    ] {
        let directory = || java_assets_atlas_SpriteSource::Directory {
            source: source.to_string(),
            prefix: sprite_prefix.to_string(),
        };
        let mut base_exists = false;
        for asset in &mut pack.assets {
            let Asset::Atlas(atlas) = asset else {
                continue;
            };
            if atlas.atlas_type != atlas_type {
                continue;
            }
            base_exists |= atlas.overlay.is_empty();
            atlas.json.sources.retain_mut(|source| match source {
                java_assets_atlas_SpriteSource::Directory { .. } => false,
                java_assets_atlas_SpriteSource::Single { resource, sprite } => {
                    let Some(relocated) = mappings.assets_mapping.get(resource) else {
                        return true;
                    };
                    let original_sprite = sprite.as_ref().unwrap_or(resource);
                    let mapped_sprite = mappings
                        .refenrence_mappings
                        .get(&original_sprite.to_string())
                        .map(|id| Identifier::parse(id));
                    if let Some(name) = relocated.path.strip_prefix(asset_prefix)
                        && let Some(mapped_sprite) = &mapped_sprite
                        && mapped_sprite.namespace == relocated.namespace
                        && mapped_sprite.path == format!("{sprite_prefix}{name}")
                    {
                        return false;
                    }

                    *sprite = Some(mapped_sprite.unwrap_or_else(|| original_sprite.clone()));
                    *resource = relocated.clone();
                    true
                }
                _ => true,
            });
            atlas.json.sources.push(directory());
        }

        if !base_exists {
            pack.assets.push(Asset::Atlas(Atlas {
                overlay: String::new(),
                json: java_assets_atlas_Atlas {
                    sources: vec![directory()],
                },
                atlas_type,
            }));
        }
    }
}

fn generate_short_name(mut id: usize, dots: bool) -> String {
    let charset = format!("abcdefghijklmnopqrstuvwxyz0123456789_-{}", if dots { "." } else { "" });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_short_name() {
        assert_eq!(generate_short_name(0, false), "a");
        assert_eq!(generate_short_name(38, true), ".");
        assert_eq!(generate_short_name(38, false), "ab");
    }
}
