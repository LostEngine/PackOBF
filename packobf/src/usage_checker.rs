use crate::minecraft::builtin_files;
use crate::resource_pack::mapping;
use crate::resource_pack::resource_pack::ResourcePack;
use crate::LogLevel::Warning;
use crate::LogMessage;
use dashmap::mapref::multiple::RefMulti;
use dashmap::DashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use tokio::sync::mpsc::UnboundedSender;

pub fn check_usage(logger: &UnboundedSender<LogMessage>, pack: &ResourcePack) {
    let counter = mapping::GLOBAL_ID_USAGE_COUNTER
        .get()
        .expect("Counter not initialized");

    check_category(
        logger,
        "Model",
        &pack.models,
        &counter.model_counter,
        |m| m.identifier.to_string(),
        |id| builtin_files::is_in_models(id),
    );

    check_category(
        logger,
        "Texture",
        &pack.textures,
        &counter.texture_counter,
        |t| t.identifier.to_string(),
        |id| builtin_files::is_in_textures(id),
    );

    check_category(
        logger,
        "Sound",
        &pack.sounds,
        &counter.sound_counter,
        |s| s.identifier.to_string(),
        |id| builtin_files::is_in_sounds(id),
    );
}

fn check_category<T>(
    logger: &UnboundedSender<LogMessage>,
    label: &str,
    pack_map: &DashMap<String, T>,
    counter_map: &DashMap<String, AtomicUsize>,
    get_id: impl Fn(&T) -> String + Sync + Send,
    is_built_in: impl Fn(&str) -> bool + Sync + Send,
) where
    T: Sync + Send + 'static,
{
    let ids_in_pack: HashSet<String> = pack_map.iter().map(|entry| get_id(entry.value())).collect();

    let ids_referenced: HashSet<String> = counter_map
        .iter()
        .map(|entry| entry.key().clone())
        .collect();

    pack_map.par_iter().for_each(|entry: RefMulti<String, T>| {
        let id = get_id(entry.value());
        if !ids_referenced.contains(&id) {
            let _ = logger.send(LogMessage {
                level: Warning,
                message: format!("Unused {}: {} (File: {})", label, id, entry.key()),
            });
        }
    });

    counter_map.par_iter().for_each(|entry| {
        let id = entry.key();
        if !is_built_in(id) && !ids_in_pack.contains(id) {
            let _ = logger.send(LogMessage {
                level: Warning,
                message: format!(
                    "Broken {} Reference: '{}' is referenced but not found in the pack",
                    label, id
                ),
            });
        }
    });
}
