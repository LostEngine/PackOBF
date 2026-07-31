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
use crate::options::{Options, ShaderCompression};
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
use crate::resource_pack::files::unknowntexture::UnknownTexture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping;
use crate::resource_pack::mapping::{IdUsageCounter, Mapping};
use crate::resource_pack::pack::ResourcePack;
use crate::LogLevel::{Info, Warning};
use dashmap::DashMap;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Sender;
use zip::ZipArchive;

pub fn process_zip(
    input_bytes: Vec<u8>,
    options: &Options,
    progress: Sender<Progress>,
    logger: &UnboundedSender<LogMessage>,
    cache_file: &Option<String>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let _ = progress.send(Progress::Idle);
    #[cfg(feature = "profiling")]
    profiler::PROFILER.store(Arc::new(profiler::Profiler::new()));

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

    let pool = ThreadPoolBuilder::new()
        .num_threads(options.num_threads.unwrap_or(0))
        .build()?;

    file_parser::parse_resource_pack_files(
        logger,
        &mut entries,
        progress.clone(),
        Arc::clone(&pack),
        &pool,
    );

    usage_checker::check_usage(logger, &pack);

    let mut mapping = Mapping::default();
    if options.rename_files {
        renamer::rename_files(logger, &pack, &mut mapping);
    }
    mapping::set_mappings(mapping);

    let mut items = collect_files(pack, &pool);

    let total = items.len();
    let mut output = Cursor::new(Vec::new());
    let writer = OptimizedZipWriter::new(&mut output);

    if options.block_unzipping {
        // Add this file first to make tools crash before they can read the data
        writer.add_file(
            "assets\0",
            Vec::new().as_slice(),
            options,
            &None,
        )?;
        // `\0` (null) is universally disallowed inside filenames, but Minecraft doesn't care
    }

    let cache = if let Some(cache) = cache_file {
        let _ = logger.send(LogMessage {
            level: Info,
            message: format!("Loading cache from {}", cache),
        });
        let cache = match Cache::load_from_file(cache) {
            Ok(cache) => {
                let _ = logger.send(LogMessage {
                    level: Info,
                    message: format!("Cache loaded: {} items", cache.items.len()),
                });
                cache
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Warning,
                    message: format!("Invalid cache file, creating a new one. Error: {}", e),
                });
                Cache {
                    items: DashMap::new(),
                }
            }
        };
        Some(cache)
    } else {
        None
    };

    pool.install(|| {
        let total_to_optimize = AtomicUsize::new(0);
        let to_optimize: Vec<_> = items
            .par_iter_mut()
            .filter(|(_, item)| match item {
                ResourcePackItem::Texture(_) => {
                    total_to_optimize.fetch_add(1, Ordering::Relaxed);
                    true
                }
                ResourcePackItem::UnknownTexture(_) => {
                    total_to_optimize.fetch_add(1, Ordering::Relaxed);
                    true
                }
                ResourcePackItem::Shader(_) => {
                    if options.shader_compression != ShaderCompression::None {
                        total_to_optimize.fetch_add(1, Ordering::Relaxed);
                        true
                    } else {
                        false
                    }
                }
                ResourcePackItem::Sound(_) => {
                    total_to_optimize.fetch_add(1, Ordering::Relaxed);
                    true
                }
                _ => false,
            })
            .collect();

        let total_to_optimize = to_optimize.len();
        let counter = AtomicUsize::new(0);
        to_optimize.into_par_iter().for_each(|(name, item)| {
            let _ = progress.send(Progress::Optimizing {
                current: name.to_string(),
                index: counter.fetch_add(1, Ordering::Relaxed),
                total: total_to_optimize,
            });
            match item {
                ResourcePackItem::Texture(x) => {
                    x.unknown_texture.optimize(options, logger, &cache);
                }
                ResourcePackItem::UnknownTexture(x) => {
                    x.optimize(options, logger, &cache);
                }
                ResourcePackItem::Shader(x) => {
                    x.optimize(options, logger);
                }
                ResourcePackItem::Sound(x) => {
                    x.optimize(logger, &cache);
                }
                _ => {}
            }
        });

        let counter = AtomicUsize::new(0);
        items.par_iter_mut().for_each(|(name, item)| {
            match add_item_to_archive(
                options, &progress, logger, total, &writer, &counter, &cache, name, item,
            ) {
                Ok(_) => {}
                Err(e) => {
                    let _ = logger.send(LogMessage {
                        level: LogLevel::Error,
                        message: format!("Failed to add item to archive: {}", e),
                    });
                }
            }
        });
    });

    writer.finish()?;

    if let Some(cache) = cache {
        #[allow(clippy::expect_used)] // Should never happen
        let _ = cache.save_to_file(cache_file.clone().expect("cache_file is None").as_str());
    }

    let _ = progress.send(Progress::Done);
    #[cfg(feature = "profiling")]
    profiler::PROFILER.load().print();
    Ok(output.into_inner())
}

#[allow(clippy::too_many_arguments)] // Generated by my IDE it's fine
fn add_item_to_archive(
    options: &Options,
    progress: &Sender<Progress>,
    logger: &UnboundedSender<LogMessage>,
    total: usize,
    writer: &OptimizedZipWriter<&mut Cursor<Vec<u8>>>,
    counter: &AtomicUsize,
    cache: &Option<Cache>,
    name: &mut String,
    item: &mut ResourcePackItem,
) -> Result<(), Box<dyn Error>> {
    let _ = progress.send(Progress::Building {
        current: name.to_string(),
        index: counter.fetch_add(1, Ordering::Relaxed),
        total,
    });
    if !name.starts_with("assets/") {
        let mut parts = name.split('/');
        let overlay = match parts.next() {
            Some(overlay) => overlay.to_string(),
            None => {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Error,
                    message: format!("Invalid file path: {}", name),
                });
                return Ok(());
            }
        };
        if let Some(value) = mapping::get_mappings().overlay_mappings.get(&overlay) {
            let rest = parts.collect::<Vec<_>>().join("/");
            *name = format!("{}/{}", value, rest);
        }
    }
    match item {
        ResourcePackItem::Texture(o) => writer.add_file(
            name.as_str(),
            o.unknown_texture.bytes.as_slice(),
            options,
            cache,
        ),
        ResourcePackItem::Shader(o) => {
            o.optimize(options, logger);
            writer.add_file(
                name.as_str(),
                o.content.as_bytes(),
                options,
                cache,
            )
        }
        ResourcePackItem::Json(o) => writer.add_file(
            name.as_str(),
            o.content.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::Model(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::Unknown(o) => writer.add_file(
            name.as_str(),
            o.bytes.as_slice(),
            options,
            cache,
        ),
        ResourcePackItem::BlockStateDefinition(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::FontDefinition(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::ItemDefinition(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::Sound(o) => writer.add_file(
            name.as_str(),
            o.bytes.as_slice(),
            options,
            cache,
        ),
        ResourcePackItem::SoundDefinitions(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::Atlas(o) => writer.add_file(
            name.as_str(),
            o.to_string().as_bytes(),
            options,
            cache,
        ),
        ResourcePackItem::UnknownTexture(o) => writer.add_file(
            name.as_str(),
            o.bytes.as_slice(),
            options,
            cache,
        ),
    }?;
    Ok(())
}

fn collect_files(
    pack: Arc<ResourcePack>,
    thread_pool: &ThreadPool,
) -> Vec<(String, ResourcePackItem)> {
    profile_scope!("collect_files");
    thread_pool.install(|| {
        let texture_iter = pack.textures.par_iter().map(|kv| {
            (
                kv.key().clone(),
                ResourcePackItem::Texture(kv.value().clone()),
            )
        });
        let unknown_texture_iter = pack.unknown_textures.par_iter().map(|kv| {
            (
                kv.key().clone(),
                ResourcePackItem::UnknownTexture(kv.value().clone()),
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
            .chain(unknown_texture_iter)
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
    })
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
    Optimizing {
        current: String,
        index: usize,
        total: usize,
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
    UnknownTexture(UnknownTexture),
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
        parts.next().unwrap_or("").to_string()
    };

    parts.next(); // skip assets

    let namespace = parts.next().unwrap_or("").to_string();

    parts.next(); // skip type

    let rest = parts.collect::<Vec<_>>().join("/");

    let (path, _) = rest.rsplit_once('.').unwrap_or(("", ""));

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
