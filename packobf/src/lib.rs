pub mod cache;
pub mod deflate;
pub mod file_parser;
pub mod minecraft;
pub mod optimized_zip_writer;
pub mod options;
pub mod overlay_remover;
pub mod profiler;
pub mod renamer;
pub mod resource_pack;
pub mod shader_minifier;
pub mod usage_checker;
pub mod utils;
pub mod version;

use crate::LogLevel::{Info, Warning};
use crate::cache::Cache;
use crate::optimized_zip_writer::OptimizedZipWriter;
use crate::options::Options;
use crate::resource_pack::files::asset_texture::AssetTexture;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::identifier::Identifier;
use crate::resource_pack::mapping;
use crate::resource_pack::mapping::{IdUsageCounter, Mapping};
use crate::resource_pack::pack::{Asset, ResourcePack};
use dashmap::DashMap;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::borrow::Cow;
use std::error::Error;
use std::io::{Cursor, Read};
use std::sync::atomic::{AtomicU32, Ordering};
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

    let mut entries = read_zip_entries(input_bytes, &progress)?;
    version::set_target_version(options.target_version.map_or(0, |version| version as u8));
    if let Some(target_version) = options.target_version {
        overlay_remover::resolve_overlays(&mut entries, target_version);
    }

    let id_usage_counter = IdUsageCounter::default();
    mapping::set_id_usage_counter(id_usage_counter);

    let pool = ThreadPoolBuilder::new()
        .num_threads(options.num_threads.unwrap_or(0))
        .build()?;

    let mut pack = file_parser::parse_resource_pack_files(logger, entries, progress.clone(), &pool);

    if options.target_version.is_some() {
        if let Some(mcmeta) = &mut pack.pack_mcmeta {
            mcmeta.overlays = None;
        }
        if version::is_older_than_1_21_4(&()) {
            pack.assets.retain(|asset| !matches!(asset, Asset::Item(_)));
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let (textures, sounds, models, atlases) = extract_usage_data(&pack);
    let logger_clone = logger.clone();
    pool.spawn_fifo(move || {
        usage_checker::check_usage(&logger_clone, textures, models, sounds, atlases);
        let _ = tx.send(());
    });

    create_mappings(options, logger, &pool, &mut pack);

    let mut output = Cursor::new(Vec::new());
    let writer = OptimizedZipWriter::new(&mut output);

    if options.block_unzipping {
        // Add this file first to make tools crash before they can read the data
        writer.add_file("\0", &[], options, &None)?;
        // `\0` (null) is universally disallowed inside filenames, but Minecraft doesn't care
    }

    let cache = load_cache(logger, cache_file);

    build_pack(options, logger, pool, pack, &writer, &cache, &progress);

    let _ = progress.send(Progress::Finishing);

    writer.finish()?;

    if let (Some(cache), Some(cache_path)) = (cache, cache_file.as_deref()) {
        let _ = cache.save_to_file(cache_path);
    }

    // Wait for check_usage to finish
    rx.recv()?;

    let _ = progress.send(Progress::Done);
    #[cfg(feature = "profiling")]
    profiler::PROFILER.load().print();
    Ok(output.into_inner())
}

fn extract_usage_data(
    pack: &ResourcePack,
) -> (
    Vec<Identifier>,
    Vec<Identifier>,
    Vec<Identifier>,
    Vec<Atlas>,
) {
    let mut textures: Vec<Identifier> = Vec::new();
    let mut sounds: Vec<Identifier> = Vec::new();
    let mut models: Vec<Identifier> = Vec::new();
    let mut atlases: Vec<Atlas> = Vec::new();
    for asset in &pack.assets {
        match asset {
            Asset::AssetTexture(x) => textures.push(x.identifier.clone()),
            Asset::Sound(x) => sounds.push(x.identifier.clone()),
            Asset::Model(x) => models.push(x.identifier.clone()),
            Asset::Atlas(x) => atlases.push(x.clone()),
            _ => {}
        }
    }
    (textures, sounds, models, atlases)
}

fn build_pack(
    options: &Options,
    logger: &UnboundedSender<LogMessage>,
    pool: ThreadPool,
    pack: ResourcePack,
    writer: &OptimizedZipWriter<&mut Cursor<Vec<u8>>>,
    cache: &Option<Cache>,
    progress: &Sender<Progress>,
) {
    macro_rules! json_file {
        ($x:expr) => {{
            let name = resolve_asset_path($x.path());
            send_progress!(name);
            (name, Cow::Owned($x.to_string().into_bytes()))
        }};
    }
    pool.install(|| {
        let total_files = pack.assets.len();

        let build_counter = AtomicU32::new(0);

        macro_rules! send_progress {
            ($name:expr) => {
                let _ = progress.send(Progress::Optimizing {
                    current: $name.to_string(),
                    index: build_counter.fetch_add(1, Ordering::Relaxed) + 1,
                    total: total_files as u32,
                });
            };
        }

        pack.assets.into_par_iter().for_each(|asset: Asset| {
            let (name, bytes): (String, Cow<'_, [u8]>) = match asset {
                Asset::AssetTexture(x) => {
                    let name = resolve_asset_path(x.path());
                    send_progress!(name);
                    let bytes = x.texture.optimize(options, logger, cache, &name);
                    (name, Cow::Owned(bytes))
                }
                Asset::UnknownTexture(x) => {
                    let name = resolve_asset_path(x.path);
                    send_progress!(name);
                    let bytes = x.texture.optimize(options, logger, cache, &name);
                    (name, Cow::Owned(bytes))
                }
                Asset::Sound(x) => {
                    let name = resolve_asset_path(x.path());
                    send_progress!(name);
                    (name, Cow::Owned(x.optimize(options, logger, cache)))
                }
                Asset::Shader(x) => {
                    let name = resolve_asset_path(x.path);
                    send_progress!(name);
                    let bytes = Shader::optimize(&name, x.content, options, logger).into_bytes();
                    (name, Cow::Owned(bytes))
                }
                Asset::Model(x) => {
                    json_file!(x)
                }
                Asset::Json(x) => {
                    let name = resolve_generic_asset_path(x.path);
                    send_progress!(name);
                    (name, Cow::Owned(x.content.to_string().into_bytes()))
                }
                Asset::Blockstate(x) => {
                    json_file!(x)
                }
                Asset::Font(x) => {
                    json_file!(x)
                }
                Asset::Item(x) => {
                    json_file!(x)
                }
                Asset::SoundDefinitions(x) => {
                    json_file!(x)
                }
                Asset::Atlas(x) => {
                    json_file!(x)
                }
                Asset::Lang(x) => {
                    json_file!(x)
                }
                Asset::UnknownFile(x) => {
                    let name = resolve_generic_asset_path(x.path);
                    send_progress!(name);
                    (name, Cow::Owned(x.bytes))
                }
            };

            if let Err(e) = writer.add_file(&name, &bytes, options, cache) {
                let _ = logger.send(LogMessage {
                    level: LogLevel::Error,
                    message: format!("Failed to add item {name} to archive: {e}"),
                });
            }
        });

        if let Some(ref mcmeta) = pack.pack_mcmeta {
            let _ = writer.add_file(mcmeta.path(), mcmeta.to_string().as_bytes(), options, cache);
        }
    });
}

fn load_cache(logger: &UnboundedSender<LogMessage>, cache_file: &Option<String>) -> Option<Cache> {
    cache_file.as_ref().map_or_else(
        || None,
        |cache| {
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
        },
    )
}

fn create_mappings(
    options: &Options,
    logger: &UnboundedSender<LogMessage>,
    pool: &ThreadPool,
    pack: &mut ResourcePack,
) {
    let mut mapping = Mapping::default();
    if options.rename_files {
        renamer::rename_files(logger, pack, &mut mapping, pool);
    }
    mapping::set_mappings(mapping);
}

fn resolve_asset_path(path: String) -> String {
    if !path.starts_with("assets/")
        && let Some((overlay, rest)) = path.split_once('/')
        && let Some(value) = mapping::get_mappings().overlay_mappings.get(overlay)
    {
        return format!("{value}/{rest}");
    }
    path
}

fn resolve_generic_asset_path(path: String) -> String {
    let path = if let Some(png_path) = path.strip_suffix(".mcmeta")
        && png_path.ends_with(".png")
        && get_type(png_path) == Some("textures")
    {
        let (overlay, identifier) = parse_path(png_path);
        format!("{}.mcmeta", AssetTexture::path_for(&overlay, &identifier))
    } else {
        path
    };
    resolve_asset_path(path)
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

        let name = file.name()?.to_string();
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
    Finishing,
    Done,
}

fn get_type(path: &str) -> Option<&str> {
    let mut parts = path.split('/');
    if path.starts_with("assets/") {
        parts.nth(2)
    } else if parts.nth(1) == Some("assets") {
        parts.nth(1)
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

    (
        overlay.to_string(),
        Identifier::new(namespace.to_string(), path.to_string()),
    )
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
        assert_eq!(id, Identifier::new("minecraft", "block/stone"));
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
