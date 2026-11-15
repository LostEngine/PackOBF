use crate::resource_pack::files::pack_mcmeta::{
    FormatRange, OverlayEntry, PackMcmeta, PackVersion,
};
use crate::version::MinecraftVersion;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub fn resolve_overlays(entries: &mut Vec<(String, Vec<u8>)>, target_version: MinecraftVersion) {
    let Some(overlays) = entries
        .iter()
        .rev()
        .filter(|(path, _)| path == "pack.mcmeta")
        .find_map(|(_, bytes)| serde_json::from_slice::<PackMcmeta>(bytes).ok())
        .and_then(|mcmeta| mcmeta.overlays)
        .and_then(|overlays| overlays.entries)
        .filter(|overlays| !overlays.is_empty())
    else {
        return;
    };

    let mut priorities = HashMap::new();
    for (index, overlay) in overlays.iter().enumerate() {
        let priority = priorities.entry(overlay.directory.as_str()).or_insert(None);
        if entry_matches(overlay, target_version as i32) {
            *priority = Some(index + 1);
        }
    }

    let mut resolved: HashMap<String, (usize, Vec<u8>)> = HashMap::with_capacity(entries.len());
    for (path, bytes) in entries.drain(..) {
        let overlay = path.match_indices('/').rev().find_map(|(index, _)| {
            priorities
                .get(&path[..index])
                .map(|priority| (&path[index + 1..], *priority))
        });
        let (path, priority) = match overlay {
            Some(("pack.mcmeta", _) | (_, None)) => continue,
            Some((path, Some(priority))) => (path.to_owned(), priority),
            None => (path, 0),
        };

        match resolved.entry(path) {
            Entry::Occupied(mut entry) if priority >= entry.get().0 => {
                entry.insert((priority, bytes));
            }
            Entry::Vacant(entry) => {
                entry.insert((priority, bytes));
            }
            Entry::Occupied(_) => {}
        }
    }

    entries.extend(resolved.into_iter().map(|(path, (_, bytes))| (path, bytes)));
}

const fn version(version: &PackVersion) -> (i32, i32) {
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
