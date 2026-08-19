use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use dashmap::DashMap;
use rayon::ThreadPool;
use tokio::sync::mpsc::UnboundedSender;

use crate::options::MinecraftVersion;
use crate::resource_pack::files::pack_mcmeta::{FormatRange, OverlayEntry, PackVersion};
use crate::resource_pack::pack::ResourcePack;
use crate::{LogLevel, LogMessage};

pub fn remove_overlays(
    logger: &UnboundedSender<LogMessage>,
    pack: &ResourcePack,
    minecraft_version: MinecraftVersion,
    thread_pool: &ThreadPool
) {
    let entries = {
        let mut mcmeta = pack.pack_mcmeta.lock().unwrap();
        let Some(mcmeta) = mcmeta.as_mut() else {
            return;
        };
        let entries = mcmeta
            .overlays
            .as_ref()
            .and_then(|overlays| overlays.entries.clone())
            .unwrap_or_default();

        // Remove all overlays from the pack
        mcmeta.overlays = None;
        entries
    };

    if entries.is_empty() {
        return;
    }

    let all: HashSet<String> = entries
        .iter()
        .map(|entry| entry.directory.clone())
        .collect();
    let active: Vec<String> = entries
        .iter()
        .filter(|entry| entry_matches(entry, minecraft_version as i32))
        .map(|entry| entry.directory.clone())
        .collect();

    #[allow(unused_mut)]
    let mut changed = AtomicUsize::new(0);
    macro_rules! resolve_typed {
        ($map:expr) => {
            changed.fetch_add(
                resolve_map(
                    $map,
                    &all,
                    &active,
                    |value| value.overlay.clone(),
                    |value| {
                        value.overlay.clear();
                        value.path()
                    },
                ),
                Ordering::Relaxed
            );
        };
    }

    macro_rules! resolve_path {
        ($map:expr) => {
            changed.fetch_add(
                resolve_map(
                    $map,
                    &all,
                    &active,
                    |value| overlay_from_path(&value.path, &all),
                    |value| {
                        value.path = strip_overlay(&value.path).to_owned();
                        value.path.clone()
                    },
                ),
                Ordering::Relaxed
            );
        };
    }

    thread_pool.install(|| {
        rayon::scope(|s| {
            s.spawn(|_| {
                resolve_typed!(&pack.models);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.textures);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.blockstates);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.fonts);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.items);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.sounds);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.sound_definitions);
            });
            s.spawn(|_| {
                resolve_typed!(&pack.atlases);
            });
            s.spawn(|_| {
                resolve_path!(&pack.json_files);
            });
            s.spawn(|_| {
                resolve_path!(&pack.shaders);
            });
            s.spawn(|_| {
                resolve_path!(&pack.unknown_textures);
            });
            s.spawn(|_| {
                resolve_path!(&pack.unknown_files);
            });
        });
    });

    let changed = changed.load(Ordering::Relaxed);
    let _ = logger.send(LogMessage {
        level: LogLevel::Info,
        message: format!(
            "Removed resource pack overlays for pack format {} ({} files changed)",
            minecraft_version as i32, changed
        ),
    });
}

fn resolve_map<T: Clone>(
    map: &DashMap<String, T>,
    declared: &HashSet<String>,
    active: &[String],
    overlay: impl Fn(&T) -> String,
    promote: impl Fn(&mut T) -> String,
) -> usize {
    let entries: Vec<(String, T, String)> = map
        .iter()
        .map(|item| {
            let value = item.value().clone();
            (item.key().clone(), value.clone(), overlay(&value))
        })
        .filter(|(_, _, directory)| declared.contains(directory))
        .collect();

    for (key, _, _) in &entries {
        map.remove(key);
    }

    for directory in active {
        for (_, mut value, entry_overlay) in entries.iter().cloned() {
            if &entry_overlay == directory {
                let path = promote(&mut value);
                map.insert(path, value);
            }
        }
    }

    entries.len()
}

fn overlay_from_path(path: &str, declared: &HashSet<String>) -> String {
    declared
        .iter()
        .find(|directory| path.starts_with(&format!("{directory}/")))
        .cloned()
        .unwrap_or_default()
}

fn strip_overlay(path: &str) -> &str {
    path.split_once('/').map_or(path, |(_, rest)| rest)
}

fn version(version: &PackVersion) -> (i32, i32) {
    match version {
        PackVersion::Integer(major) => (*major, 0),
        PackVersion::Decimal([major, minor]) => (*major, *minor),
    }
}

fn entry_matches(entry: &OverlayEntry, target_major: i32) -> bool {
    let target = (target_major, 0);
    if let Some(formats) = &entry.formats {
        return match formats {
            FormatRange::Int(exact) => target == (*exact, 0),
            FormatRange::List([min, max]) => (*min, 0) <= target && target <= (*max, 0),
            FormatRange::Object {
                min_inclusive,
                max_inclusive,
                ..
            } => version(min_inclusive) <= target && target <= version(max_inclusive),
        };
    }

    entry
        .min_format
        .as_ref()
        .is_none_or(|min| version(min) <= target)
        && entry
            .max_format
            .as_ref()
            .is_none_or(|max| target <= version(max))
}
