pub mod cache;
pub mod file_parser;
pub mod minecraft;
pub mod optimized_zip_writer;
pub mod options;
pub mod profiler;
pub mod renamer;
pub mod resource_pack;
pub mod shader_minifier;
pub mod usage_checker;
pub mod utils;
pub mod overlay_remover;
pub mod version;
pub mod deflate;

use crate::cache::Cache;
use crate::optimized_zip_writer::OptimizedZipWriter;
use crate::options::{Options, ShaderCompression};
use crate::resource_pack::files::asset_texture::AssetTexture;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sound_definitions::SoundDefinitions;
use crate::resource_pack::files::unknowntexture::UnknownTexture;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping;
use crate::resource_pack::mapping::{IdUsageCounter, Mapping};
use crate::resource_pack::pack::ResourcePack;
use crate::LogLevel::{Info, Warning};
use dashmap::DashMap;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::borrow::Cow;
use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Sender;
use zip::ZipArchive;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn process_zip(
    input_bytes: Vec<u8>,
    options: &Options,
    progress: Sender<Progress>,
    logger: &UnboundedSender<LogMessage>,
    cache_file: &Option<String>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    let _ = progress.send(Progress::Idle);
    #[cfg(feature = "profiling")]
    profiler::PROFILER.store(Arc::new(profiler::Profiler::new()));

    let entries = read_zip_entries(input_bytes, &progress)?;

    let pack = Arc::new(ResourcePack::default());

    let id_usage_counter = IdUsageCounter::default();
    mapping::set_id_usage_counter(id_usage_counter);

    let pool = ThreadPoolBuilder::new()
        .num_threads(options.num_threads.unwrap_or(0))
        .build()?;

    file_parser::parse_resource_pack_files(
        logger,
        entries,
        progress.clone(),
        Arc::clone(&pack),
        &pool,
    );

    let pack = Arc::try_unwrap(pack).unwrap_or_else(|arc| (*arc).clone());
    let mut pack = pack.freeze();

    if let Some(target_version) = options.target_version {
        version::set_target_version(target_version as u8);
        overlay_remover::remove_overlays(logger, &mut pack, target_version, &pool);
        if version::is_older_than_1_21_4(&()) {
            pack.items.clear(); // Added in 1.21.4
        }
    } else {
        version::set_target_version(0);
    }

    usage_checker::check_usage(logger, &pack);

    let mut mapping = Mapping::default();
    if options.rename_files {
        renamer::rename_files(logger, &mut pack, &mut mapping);
    }
    mapping::set_mappings(mapping);

    let mut output = Cursor::new(Vec::new());
    let writer = OptimizedZipWriter::new(&mut output);

    if options.block_unzipping {
        // Add this file first to make tools crash before they can read the data
        writer.add_file("assets\0", &[], options, &None)?;
        // `\0` (null) is universally disallowed inside filenames, but Minecraft doesn't care
    }

    let cache = cache_file.as_ref().map_or_else(|| None, |cache| {
        let _ = logger.send(LogMessage {
            level: Info,
            message: format!("Loading cache from {cache}"),
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
                    message: format!("Invalid cache file, creating a new one. Error: {e}"),
                });
                Cache {
                    items: DashMap::new(),
                }
            }
        };
        Some(cache)
    });

    pool.install(|| {
        let optimize_shaders = options.shader_compression != ShaderCompression::None;

        let total_to_optimize = pack.textures.len()
            + pack.unknown_textures.len()
            + pack.sounds.len()
            + if optimize_shaders { pack.shaders.len() } else { 0 };

        let total_files = pack.textures.len()
            + pack.unknown_textures.len()
            + pack.sounds.len()
            + pack.shaders.len()
            + pack.models.len()
            + pack.json_files.len()
            + pack.unknown_files.len()
            + pack.blockstates.len()
            + pack.fonts.len()
            + pack.items.len()
            + pack.sound_definitions.len()
            + pack.atlases.len();

        let opt_counter = AtomicU32::new(0);
        let build_counter = AtomicU32::new(0);

        enum PackItem {
            AssetTexture(String, AssetTexture),
            UnknownTexture(String, UnknownTexture),
            Sound(String, Sound),
            Shader(String, Shader),
            Model(String, Model),
            Json(String, Json),
            Blockstate(String, Blockstate),
            Font(String, Font),
            Item(String, Item),
            SoundDefinitions(String, SoundDefinitions),
            Atlas(String, Atlas),
            GenericFile(String, Vec<u8>),
        }

        let items = std::iter::empty()
            .chain(pack.textures.into_iter().map(|(n, t)| PackItem::AssetTexture(n, t)))
            .chain(pack.unknown_textures.into_iter().map(|(n, ut)| PackItem::UnknownTexture(n, ut)))
            .chain(pack.sounds.into_iter().map(|(n, s)| PackItem::Sound(n, s)))
            .chain(pack.shaders.into_iter().map(|(n, s)| if optimize_shaders {
                PackItem::Shader(n, s)
            } else {
                PackItem::GenericFile(n, s.content.into_bytes())
            }))
            .chain(pack.models.into_iter().map(|(n, m)| PackItem::Model(n, m)))
            .chain(pack.json_files.into_iter().map(|(n, j)| PackItem::Json(n, j)))
            .chain(pack.blockstates.into_iter().map(|(n, b)| PackItem::Blockstate(n, b)))
            .chain(pack.fonts.into_iter().map(|(n, f)| PackItem::Font(n, f)))
            .chain(pack.items.into_iter().map(|(n, i)| PackItem::Item(n, i)))
            .chain(pack.sound_definitions.into_iter().map(|(n, sd)| PackItem::SoundDefinitions(n, sd)))
            .chain(pack.atlases.into_iter().map(|(n, a)| PackItem::Atlas(n, a)))
            .chain(pack.unknown_files.into_iter().map(|(n, u)| PackItem::GenericFile(n, u.bytes)));

        items.par_bridge().for_each(|item| {
            let (raw_path, bytes): (String, Cow<'_, [u8]>) = match item {
                PackItem::AssetTexture(name, t) => {
                    let _ = progress.send(Progress::Optimizing {
                        current: name.clone(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize as u32,
                    });
                    (name, Cow::Owned(t.optimize(options, logger, &cache)))
                }
                PackItem::UnknownTexture(name, ut) => {
                    let _ = progress.send(Progress::Optimizing {
                        current: name.clone(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize as u32,
                    });
                    (name, Cow::Owned(ut.optimize(options, logger, &cache)))
                }
                PackItem::Sound(name, s) => {
                    let _ = progress.send(Progress::Optimizing {
                        current: name.clone(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize as u32,
                    });
                    (name, Cow::Owned(s.optimize(logger, &cache)))
                }
                PackItem::Shader(name, s) => {
                    if optimize_shaders {
                        let _ = progress.send(Progress::Optimizing {
                            current: name.clone(),
                            index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                            total: total_to_optimize as u32,
                        });
                    }
                    (name, Cow::Owned(s.optimize(options, logger).into_bytes()))
                }
                PackItem::Model(name, m) => (name, Cow::Owned(m.to_string().into_bytes())),
                PackItem::Json(name, j) => (name, Cow::Owned(j.content.to_string().into_bytes())),
                PackItem::Blockstate(name, b) => (name, Cow::Owned(b.to_string().into_bytes())),
                PackItem::Font(name, f) => (name, Cow::Owned(f.to_string().into_bytes())),
                PackItem::Item(name, i) => (name, Cow::Owned(i.to_string().into_bytes())),
                PackItem::SoundDefinitions(name, sd) => (name, Cow::Owned(sd.to_string().into_bytes())),
                PackItem::Atlas(name, a) => (name, Cow::Owned(a.to_string().into_bytes())),
                PackItem::GenericFile(name, u) => (name, Cow::Owned(u)),
            };

            let target_path = resolve_asset_path(&raw_path);

            let _ = progress.send(Progress::Building {
                current: target_path.as_ref().to_string(),
                index: build_counter.fetch_add(1, Ordering::Relaxed) + 1,
                total: total_files as u32,
            });

            if let Err(e) = writer.add_file(&target_path, &bytes, options, &cache) {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Error,
                    message: format!("Failed to add item {target_path} to archive: {e}"),
                });
            }
        });

        if let Some(ref mcmeta) = pack.pack_mcmeta {
            let _ = writer.add_file(mcmeta.path(), mcmeta.to_string().as_bytes(), options, &cache);
        }
    });

    writer.finish()?;

    if let (Some(cache), Some(cache_path)) = (cache, cache_file.as_deref()) {
        let _ = cache.save_to_file(cache_path);
    }

    let _ = progress.send(Progress::Done);
    #[cfg(feature = "profiling")]
    profiler::PROFILER.load().print();
    Ok(output.into_inner())
}

fn resolve_asset_path(path: &str) -> Cow<'_, str> {
    if !path.starts_with("assets/") {
        if let Some((overlay, rest)) = path.split_once('/') {
            if let Some(value) = mapping::get_mappings().overlay_mappings.get(overlay) {
                return Cow::Owned(format!("{value}/{rest}"));
            }
        }
    }
    Cow::Borrowed(path)
}

fn read_zip_entries(
    input_bytes: Vec<u8>,
    progress: &Sender<Progress>,
) -> Result<Vec<(String, Vec<u8>)>, Box<dyn Error + Send + Sync>> {
    let reader = Cursor::new(input_bytes);
    let mut archive = ZipArchive::new(reader)?;

    let len = archive.len();
    let mut entries = Vec::with_capacity(len);

    for i in 0..len {
        let _ = progress.send(Progress::ReadingZip {
            current: i as u32,
            total: len as u32,
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

    Ok(entries)
}

#[derive(Clone, Debug)]
pub enum Progress {
    Idle,
    ReadingZip {
        current: u32,
        total: u32,
    },
    Parsing {
        current: String,
    },
    Optimizing {
        current: String,
        index: u32,
        total: u32,
    },
    Building {
        current: String,
        index: u32,
        total: u32,
    },
    Done,
}

fn get_type(path: &str) -> Option<&str> {
    let mut parts = path.split('/');
    if path.starts_with("assets/") {
        parts.nth(2)
    } else if parts.nth(2) == Some("assets") {
        parts.next()
    } else {
        None
    }
}

fn parse_path(path: &str) -> (String, Identifier) {
    let (overlay, rem) = if path.starts_with("assets/") {
        ("", path)
    } else {
        let mut parts = path.splitn(2, '/');
        (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
    };

    let mut parts = rem.splitn(4, '/');
    parts.next(); // skip assets
    let namespace = parts.next().unwrap_or("");
    parts.next(); // skip type
    let rest = parts.next().unwrap_or("");

    let (path, _) = rest.rsplit_once('.').unwrap_or((rest, ""));

    (overlay.to_string(), Identifier::new(namespace.to_string(), path.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_type() {
        assert_eq!(
            get_type("assets/minecraft/textures/block/stone.png"),
            Some("textures")
        );
        assert_eq!(
            get_type("assets/minecraft/models/item/tool.json"),
            Some("models")
        );
        assert_eq!(
            get_type("overlay_1/assets/minecraft/textures/block/dirt.png"),
            Some("textures")
        );
        assert_eq!(get_type(""), None);
        assert_eq!(get_type("assets/"), None);
        assert_eq!(get_type("assets/minecraft"), None);
    }

    #[test]
    fn test_parse_path() {
        let (overlay, id) = parse_path("assets/minecraft/textures/block/stone.png");
        assert_eq!(overlay, "");
        assert_eq!(
            id,
            Identifier::new("minecraft", "block/stone")
        );
        let (overlay, id) = parse_path("overlay_1/assets/minecraft/models/item/stick.json");
        assert_eq!(overlay, "overlay_1");
        assert_eq!(
            id,
            Identifier::new("minecraft".to_string(), "item/stick".to_string())
        );
        let (overlay, id) = parse_path("assets/minecraft/textures/block/my.block.png");
        assert_eq!(overlay, "");
        assert_eq!(
            id,
            Identifier::new("minecraft".to_string(), "block/my.block".to_string())
        );
        let (overlay, id) = parse_path("assets/minecraft/textures/no_extension");
        assert_eq!(overlay, "");
        assert_eq!(
            id,
            Identifier::new("minecraft".to_string(), "no_extension".to_string())
        );
    }
}
