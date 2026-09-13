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
pub mod overlay_remover;
pub mod version;

use crate::cache::Cache;
use crate::optimized_zip_writer::OptimizedZipWriter;
use crate::options::{Options, ShaderCompression};
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
use std::sync::atomic::{AtomicUsize, Ordering};
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

    let cache = if let Some(cache) = cache_file {
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
    } else {
        None
    };

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

        let off_unkn_tex = pack.textures.len();
        let off_sounds = off_unkn_tex + pack.unknown_textures.len();
        let off_shaders = off_sounds + pack.sounds.len();
        let off_models = off_shaders + pack.shaders.len();
        let off_jsons = off_models + pack.models.len();
        let off_blockstates = off_jsons + pack.json_files.len();
        let off_fonts = off_blockstates + pack.blockstates.len();
        let off_items = off_fonts + pack.fonts.len();
        let off_sound_defs = off_items + pack.items.len();
        let off_atlases = off_sound_defs + pack.sound_definitions.len();
        let off_unknown_files = off_atlases + pack.atlases.len();

        let tex_slice = pack.textures.as_mut_slice();
        let unkn_tex_slice = pack.unknown_textures.as_mut_slice();
        let sounds_slice = pack.sounds.as_mut_slice();
        let shaders_slice = pack.shaders.as_mut_slice();
        let models_slice = pack.models.as_slice();
        let jsons_slice = pack.json_files.as_slice();
        let blockstates_slice = pack.blockstates.as_slice();
        let fonts_slice = pack.fonts.as_slice();
        let items_slice = pack.items.as_slice();
        let sound_defs_slice = pack.sound_definitions.as_slice();
        let atlases_slice = pack.atlases.as_slice();
        let unknown_files_slice = pack.unknown_files.as_slice();

        let boundaries = [
            off_unkn_tex,
            off_sounds,
            off_shaders,
            off_models,
            off_jsons,
            off_blockstates,
            off_fonts,
            off_items,
            off_sound_defs,
            off_atlases,
            off_unknown_files,
        ];

        let opt_counter = AtomicUsize::new(0);
        let build_counter = AtomicUsize::new(0);
        let i = AtomicUsize::new(0);

        (0..total_files).into_par_iter().for_each(|_| {
            let i = i.fetch_add(1, Ordering::Relaxed);
            let category = boundaries.partition_point(|&b| i >= b);
            let (raw_path, bytes): (&str, Cow<'_, [u8]>) = match category {
                0 => {
                    let (name, t) = &tex_slice[i];
                    let _ = progress.send(Progress::Optimizing {
                        current: name.to_string(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize,
                    });
                    (name.as_str(), Cow::Owned(t.optimize(options, logger, &cache)))
                }
                1 => {
                    let local_i = i - off_unkn_tex;
                    let (name, ut) = &unkn_tex_slice[local_i];
                    let _ = progress.send(Progress::Optimizing {
                        current: name.to_string(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize,
                    });
                    (name.as_str(), Cow::Owned(ut.optimize(options, logger, &cache)))
                }
                2 => {
                    let local_i = i - off_sounds;
                    let (name, s) = &sounds_slice[local_i];
                    let _ = progress.send(Progress::Optimizing {
                        current: name.to_string(),
                        index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                        total: total_to_optimize,
                    });
                    (name.as_str(), Cow::Owned(s.optimize(logger, &cache)))
                }
                3 => {
                    let local_i = i - off_shaders;
                    let (name, s) = &shaders_slice[local_i];
                    if optimize_shaders {
                        let _ = progress.send(Progress::Optimizing {
                            current: name.to_string(),
                            index: opt_counter.fetch_add(1, Ordering::Relaxed) + 1,
                            total: total_to_optimize,
                        });
                    }
                    (name.as_str(), Cow::Owned(s.optimize(options, logger).into_bytes()))
                }
                4 => {
                    let local_i = i - off_models;
                    let (name, m) = &models_slice[local_i];
                    (name.as_str(), Cow::Owned(m.to_string().into_bytes()))
                }
                5 => {
                    let local_i = i - off_jsons;
                    let (name, j) = &jsons_slice[local_i];
                    (name.as_str(), Cow::Owned(j.content.to_string().into_bytes()))
                }
                6 => {
                    let local_i = i - off_blockstates;
                    let (name, b) = &blockstates_slice[local_i];
                    (name.as_str(), Cow::Owned(b.to_string().into_bytes()))
                }
                7 => {
                    let local_i = i - off_fonts;
                    let (name, f) = &fonts_slice[local_i];
                    (name.as_str(), Cow::Owned(f.to_string().into_bytes()))
                }
                8 => {
                    let local_i = i - off_items;
                    let (name, item) = &items_slice[local_i];
                    (name.as_str(), Cow::Owned(item.to_string().into_bytes()))
                }
                9 => {
                    let local_i = i - off_sound_defs;
                    let (name, sd) = &sound_defs_slice[local_i];
                    (name.as_str(), Cow::Owned(sd.to_string().into_bytes()))
                }
                10 => {
                    let local_i = i - off_atlases;
                    let (name, a) = &atlases_slice[local_i];
                    (name.as_str(), Cow::Owned(a.to_string().into_bytes()))
                } _ => {
                    let local_i = i - off_unknown_files;
                    let (name, u) = &unknown_files_slice[local_i];
                    (name.as_str(), Cow::Borrowed(u.bytes.as_slice()))
                }
            };

            let target_path = resolve_asset_path(raw_path);

            let _ = progress.send(Progress::Building {
                current: target_path.as_ref().to_string(),
                index: build_counter.fetch_add(1, Ordering::Relaxed) + 1,
                total: total_files,
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

    Ok(entries)
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
