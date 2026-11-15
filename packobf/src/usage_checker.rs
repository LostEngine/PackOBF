use crate::LogLevel::Warning;
use crate::minecraft::builtin_files;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping;
use crate::{LogMessage, profile_scope};
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use tokio::sync::mpsc::UnboundedSender;

pub fn check_usage(
    logger: &UnboundedSender<LogMessage>,
    textures: Vec<Identifier>,
    models: Vec<Identifier>,
    sounds: Vec<Identifier>,
    atlases: Vec<Atlas>,
) {
    profile_scope!(std::any::type_name_of_val(&check_usage));
    let counter = mapping::get_id_usage_counter();

    rayon::scope(|s| {
        s.spawn(|_| check_category(
            logger,
            "Model",
            &models,
            &counter.model_counter,
            builtin_files::is_in_models,
            |m, ids_referenced| ids_referenced.contains(m),
            |id| models.contains(id),
        ));
        s.spawn(|_| check_category(
            logger,
            "Texture",
            &textures,
            &counter.texture_counter,
            |id| {
                id.starts_with("#") || builtin_files::is_in_textures(id)
            },
            |t, ids_referenced| {
                ids_referenced.contains(t)
                    || atlases.iter().any(|atlas| {
                        atlas.get_sprites(t).iter().any(|sprite| {
                            ids_referenced.contains(sprite)
                        })
                    })
            },
            |id| {
                textures.contains(id)
                    || atlases.iter().any(|atlas| {
                        atlas.has_declared_sprite(id)
                    })
            },
        ));
        check_category(
            logger,
            "Sound",
            &sounds,
            &counter.sound_counter,
            builtin_files::is_in_sounds,
            |s, ids_referenced| ids_referenced.contains(s),
            |id| sounds.contains(id),
        );
    });
}

fn check_category(
    logger: &UnboundedSender<LogMessage>,
    label: &str,
    vec: &Vec<Identifier>,
    counter_map: &DashMap<Identifier, AtomicUsize>,
    is_built_in: impl Fn(&str) -> bool + Sync + Send,
    is_referenced: impl Fn(&Identifier, &HashSet<Identifier>) -> bool + Sync + Send,
    is_present: impl Fn(&Identifier) -> bool + Sync + Send,
) where
    Identifier: Sync + Send + 'static,
{
    let ids_referenced: HashSet<Identifier> = counter_map
        .iter()
        .map(|entry| entry.key().clone())
        .collect();

    for id in vec {
        if !is_built_in(&id.to_string()) && !is_referenced(id, &ids_referenced) {
            let _ = logger.send(LogMessage {
                level: Warning,
                message: format!("Unused {}: {}", label, id),
            });
        }
    }

    for entry in counter_map {
        let id = entry.key();
        if !is_built_in(&id.to_string()) && !is_present(id) {
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
