use crate::minecraft::builtin_files;
use crate::resource_pack::mapping;
use crate::resource_pack::pack::FrozenResourcePack;
use crate::LogLevel::Warning;
use crate::{profile_scope, LogMessage};
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use tokio::sync::mpsc::UnboundedSender;

pub fn check_usage(logger: &UnboundedSender<LogMessage>, pack: &FrozenResourcePack) {
    profile_scope!(std::any::type_name_of_val(&check_usage));
    let counter = mapping::get_id_usage_counter();

    rayon::scope(|s| {
        s.spawn(|_| check_category(
            logger,
            "Model",
            &pack.models,
            &counter.model_counter,
            |m| m.identifier.to_string(),
            builtin_files::is_in_models,
            |m, referenced| referenced.contains(&m.identifier.to_string()),
            |id, ids| ids.contains(id),
        ));
        s.spawn(|_| check_category(
            logger,
            "Texture",
            &pack.textures,
            &counter.texture_counter,
            |t| t.identifier.to_string(),
            builtin_files::is_in_textures,
            |t, referenced| {
                referenced.contains(&t.identifier.to_string())
                    || pack.atlases.iter().any(|(_, atlas)| {
                        atlas.get_sprites(&t.identifier).iter().any(|sprite| {
                            referenced.contains(&sprite.to_string())
                        })
                    })
            },
            |id, ids| {
                ids.contains(id)
                    || pack.textures.iter().any(|(_, texture)| {
                        atlas_has_sprite(pack, &texture.identifier, id)
                    })
            },
        ));
        check_category(
            logger,
            "Sound",
            &pack.sounds,
            &counter.sound_counter,
            |s| s.identifier.to_string(),
            builtin_files::is_in_sounds,
            |s, referenced| referenced.contains(&s.identifier.to_string()),
            |id, ids| ids.contains(id),
        );
    });
}

fn check_category<T>(
    logger: &UnboundedSender<LogMessage>,
    label: &str,
    vec: &Vec<(String, T)>,
    counter_map: &DashMap<String, AtomicUsize>,
    get_id: impl Fn(&T) -> String + Sync + Send,
    is_built_in: impl Fn(&str) -> bool + Sync + Send,
    is_referenced: impl Fn(&T, &HashSet<String>) -> bool + Sync + Send,
    is_present: impl Fn(&str, &HashSet<String>) -> bool + Sync + Send,
) where
    T: Sync + Send + 'static,
{
    let ids_in_pack: HashSet<String> = vec.iter().map(|entry| get_id(&entry.1)).collect();

    let ids_referenced: HashSet<String> = counter_map
        .iter()
        .map(|entry| entry.key().clone())
        .collect();

    for (path, item) in vec {
        let id = get_id(item);
        if !is_referenced(item, &ids_referenced) {
            let _ = logger.send(LogMessage {
                level: Warning,
                message: format!("Unused {}: {} (File: {})", label, id, path),
            });
        }
    }

    for entry in counter_map {
        let id = entry.key();
        if !is_built_in(id) && !is_present(id, &ids_in_pack) {
            let _ = logger.send(LogMessage {
                level: Warning,
                message: format!(
                    "Broken {} Reference: '{}' is referenced but not found in the pack",
                    label, id
                ),
            });
        }
    }
}

fn atlas_has_sprite(pack: &FrozenResourcePack, texture_id: &crate::resource_pack::identifier::Identifier, id: &str) -> bool {
    pack.atlases.iter().any(|(_, atlas)| {
        atlas.get_sprites(texture_id).iter().any(|sprite| sprite.to_string() == id)
    })
}
