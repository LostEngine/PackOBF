pub mod cache;
pub mod file_parser;
pub mod minecraft;
pub mod optimized_zip_writer;
pub mod options;
pub mod png;
pub mod profiler;
pub mod renamer;
pub mod resource_pack;
pub mod shader_minifier;
pub mod usage_checker;
pub mod utils;

use crate::cache::Cache;
use crate::optimized_zip_writer::OptimizedZipWriter;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::resource_pack_file::ResourcePackFile;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sound_definitions::SoundDefinitions;
use crate::resource_pack::files::texture::Texture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping;
use crate::resource_pack::mapping::{IdUsageCounter, Mapping};
use crate::resource_pack::resource_pack::ResourcePack;
use crate::LogLevel::Info;
use rayon::prelude::*;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use zip::ZipArchive;

pub fn process_zip(
    input_bytes: Vec<u8>,
    options: &options::Options,
    progress: watch::Sender<Progress>,
    logger: &tokio::sync::mpsc::UnboundedSender<LogMessage>,
    cache_file: &Option<String>,
) -> zip::result::ZipResult<Vec<u8>> {
    let _ = progress.send(Progress::Idle);
    #[cfg(feature = "profiling")]
    profiler::profiler::PROFILER.store(Arc::new(profiler::profiler::Profiler::new()));

    let progress_clone = progress.clone();
    let reader = Cursor::new(&input_bytes);
    let mut archive = ZipArchive::new(reader)?;

    let len = archive.len();
    let mut entries = Vec::with_capacity(len);

    for i in 0..len {
        let _ = progress_clone.send(Progress::ReadingZip {
            current: i,
            total: len,
        });
        let mut file = archive.by_index(i)?;

        if file.is_dir() {
            continue;
        }

        let name = file.name().to_string();
        let mut content = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut content)?;

        entries.push((name, content));
    }

    let pack = Arc::new(ResourcePack::default());

    let id_usage_counter = IdUsageCounter::default();
    mapping::set_id_usage_counter(id_usage_counter);

    file_parser::parse_resource_pack_files(
        logger,
        &mut entries,
        progress.clone(),
        Arc::clone(&pack),
    );

    usage_checker::check_usage(logger, &pack);

    let mut mapping = Mapping::default();
    if options.rename_files {
        renamer::rename_files(logger, &pack, &mut mapping);
    }
    mapping::set_mappings(mapping);

    let mut items = collect_files(pack);

    let total = items.len();
    let mut output = Cursor::new(Vec::new());
    let writer = OptimizedZipWriter::new(&mut output);
    let counter = AtomicUsize::new(0);

    if options.block_unzipping {
        // Add this file first to make tools crash before they can read the data
        writer.add_file("assets\0", Vec::new().as_slice(), &options, &None)?;
        // `\0` (null) is universally disallowed inside filenames, but Minecraft doesn't care
    }

    let cache = if let Some(cache) = cache_file {
        let _ = logger.send(LogMessage {
            level: Info,
            message: format!("Loading cache from {}", cache),
        });
        let cache = Cache::load_from_file(cache)?;
        let _ = logger.send(LogMessage {
            level: Info,
            message: format!("Cache loaded: {} items", cache.items.len()),
        });
        Some(cache)
    } else {
        None
    };

    items.par_iter_mut().for_each(|(name, item)| {
        let _ = progress.send(Progress::Building {
            current: name.to_string(),
            index: counter.fetch_add(1, Ordering::Relaxed),
            total,
        });
        if !name.starts_with("assets/") {
            let mut parts = name.split('/');
            let overlay = parts.next().unwrap().to_string();
            if let Some(value) = mapping::get_mappings().overlay_mappings.get(&overlay) {
                let rest = parts.collect::<Vec<_>>().join("/");
                *name = format!("{}/{}", value, rest);
            }
        }
        match item {
            ResourcePackItem::Texture(t) => t.optimize(&options, &logger, &cache),
            ResourcePackItem::Shader(s) => s.optimize(&options, &logger),
            ResourcePackItem::Sound(s) => s.optimize(&logger, &cache),
            _ => {}
        }
        match item {
            ResourcePackItem::Texture(o) => {
                writer.add_file(name.as_str(), o.bytes.as_slice(), &options, &cache)
            }
            ResourcePackItem::Shader(o) => {
                writer.add_file(name.as_str(), o.content.as_bytes(), &options, &cache)
            }
            ResourcePackItem::Json(o) => writer.add_file(
                name.as_str(),
                o.content.to_string().as_bytes(),
                &options,
                &cache,
            ),
            ResourcePackItem::Model(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
            ResourcePackItem::Unknown(o) => {
                writer.add_file(name.as_str(), o.bytes.as_slice(), &options, &cache)
            }
            ResourcePackItem::BlockStateDefinition(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
            ResourcePackItem::FontDefinition(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
            ResourcePackItem::ItemDefinition(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
            ResourcePackItem::Sound(o) => {
                writer.add_file(name.as_str(), o.bytes.as_slice(), &options, &cache)
            }
            ResourcePackItem::SoundDefinitions(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
            ResourcePackItem::Atlas(o) => {
                writer.add_file(name.as_str(), o.to_string().as_bytes(), &options, &cache)
            }
        }
        .expect("Failed to write file to zip archive");
    });

    writer.finish()?;

    if let Some(cache) = cache {
        let _ = cache.save_to_file(cache_file.clone().unwrap().as_str());
    }

    let _ = progress.send(Progress::Done);
    #[cfg(feature = "profiling")]
    profiler::profiler::PROFILER.load().print();
    Ok(output.into_inner())
}

fn collect_files(pack: Arc<ResourcePack>) -> Vec<(String, ResourcePackItem)> {
    profile_scope!("collect_files");
    let texture_iter = pack.textures.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Texture(kv.value().clone()),
        )
    });
    let shader_iter = pack.shaders.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Shader(kv.value().clone()),
        )
    });
    let model_iter = pack.models.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Model(kv.value().clone()),
        )
    });
    let json_iter = pack
        .json_files
        .par_iter()
        .map(|kv| (kv.key().clone(), ResourcePackItem::Json(kv.value().clone())));
    let unknown_iter = pack.unknown_files.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Unknown(kv.value().clone()),
        )
    });
    let blockstate_iter = pack.blockstates.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::BlockStateDefinition(kv.value().clone()),
        )
    });
    let font_iter = pack.fonts.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::FontDefinition(kv.value().clone()),
        )
    });
    let item_iter = pack.items.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::ItemDefinition(kv.value().clone()),
        )
    });
    let sound_iter = pack.sounds.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Sound(kv.value().clone()),
        )
    });
    let sound_definitions_iter = pack.sound_definitions.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::SoundDefinitions(kv.value().clone()),
        )
    });
    let atlas_iter = pack.atlases.par_iter().map(|kv| {
        (
            kv.key().clone(),
            ResourcePackItem::Atlas(kv.value().clone()),
        )
    });

    texture_iter
        .chain(shader_iter)
        .chain(model_iter)
        .chain(json_iter)
        .chain(unknown_iter)
        .chain(blockstate_iter)
        .chain(font_iter)
        .chain(item_iter)
        .chain(sound_iter)
        .chain(sound_definitions_iter)
        .chain(atlas_iter)
        .collect()
}

#[derive(Clone, Debug)]
pub enum Progress {
    Idle,
    ReadingZip {
        current: usize,
        total: usize,
    },
    Parsing {
        current: String,
    },
    Building {
        current: String,
        index: usize,
        total: usize,
    },
    Done,
}

#[derive(Clone, Debug)]
enum ResourcePackItem {
    Texture(Texture),
    Shader(Shader),
    Json(Json),
    Model(Model),
    Unknown(ResourcePackFile),
    BlockStateDefinition(Blockstate),
    FontDefinition(Font),
    ItemDefinition(Item),
    Sound(Sound),
    SoundDefinitions(SoundDefinitions),
    Atlas(Atlas),
}

fn get_type(path: &str) -> Option<&str> {
    // TODO: if the overlay is assets do something
    if path.starts_with("assets/") {
        path.split('/').nth(2)
    } else {
        if path.split('/').nth(2) == Some("assets") {
            path.split('/').nth(3)
        } else {
            // If the asset's path does not contain `assets` after its overlay, we have to skip it. (e.g. `overlay/abc/textures/block/stone.png`)
            None
        }
    }
}

fn parse_path(path: &str) -> (String, Identifier) {
    let mut parts = path.split('/');

    // TODO: do something better
    let overlay = if path.starts_with("assets/") {
        "".to_string()
    } else {
        parts.next().unwrap().to_string()
    };
    parts.next().unwrap(); // skip assets
    let namespace = parts.next().unwrap().to_string();

    parts.next().unwrap(); // skip type

    let rest = parts.collect::<Vec<_>>().join("/");

    let (path, _) = rest.rsplit_once('.').or_else(|| Some(("", ""))).unwrap();

    (overlay, Identifier::new(namespace, path.to_string()))
}

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info = 0,
    Warning = 1,
    Error = 2,
}
