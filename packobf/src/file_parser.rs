use crate::minecraft::builtin_files::AtlasType;
use crate::resource_pack::files::asset_texture::AssetTexture;
use crate::resource_pack::files::atlas::Atlas;
use crate::resource_pack::files::blockstate::Blockstate;
use crate::resource_pack::files::font::Font;
use crate::resource_pack::files::item::Item;
use crate::resource_pack::files::json::Json;
use crate::resource_pack::files::lang::Lang;
use crate::resource_pack::files::model::Model;
use crate::resource_pack::files::pack_mcmeta::PackMcmeta;
use crate::resource_pack::files::shader::Shader;
use crate::resource_pack::files::sound::Sound;
use crate::resource_pack::files::sounds::SoundDefinitions;
use crate::resource_pack::files::unknown_file::UnknownFile;
use crate::resource_pack::files::unknowntexture::UnknownTexture;
use crate::resource_pack::pack::{Asset, ResourcePack};
use crate::LogLevel::{Error, Info};
use crate::{get_type, parse_path, LogMessage, Progress};
use crossbeam::queue::ArrayQueue;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::ThreadPool;
use std::str::FromStr;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Sender;

pub fn parse_resource_pack_files(
    logger: &UnboundedSender<LogMessage>,
    entries: Vec<(String, Vec<u8>)>,
    progress: Sender<Progress>,
    thread_pool: &ThreadPool
) -> ResourcePack {
    let assets: ArrayQueue<Asset> = ArrayQueue::new(entries.len().max(1));
    let pack_mcmeta: Mutex<Option<PackMcmeta>> = Mutex::new(None);

    thread_pool.install(|| {
        entries.par_iter().for_each(|(name, content)| {
            parse_resource_pack_file(logger, &progress, name, content, &assets, &pack_mcmeta);
        });
    });

    ResourcePack {
        assets: std::iter::from_fn(|| assets.pop()).collect(),
        pack_mcmeta: pack_mcmeta.into_inner().unwrap(),
    }
}

fn parse_resource_pack_file(
    logger: &UnboundedSender<LogMessage>,
    progress: &Sender<Progress>,
    name: &str,
    content: &[u8],
    assets: &ArrayQueue<Asset>,
    pack_mcmeta: &Mutex<Option<PackMcmeta>>,
) {
    let _ = progress.send(Progress::Parsing {
        current: name.to_string(),
    });
    #[cfg(feature = "profiling")]
    let _ = crate::profiler::ScopeTimer::new(
        if name.ends_with(".json") || name.ends_with(".mcmeta") {
            "parse_resource_pack_files::json"
        } else if name.ends_with(".png") && get_type(&name) == Some("textures") {
            "parse_resource_pack_files::texture"
        } else if name.ends_with(".vsh") || name.ends_with(".fsh") || name.ends_with(".glsl") {
            "parse_resource_pack_files::shader"
        } else if name.ends_with(".ogg") && get_type(&name) == Some("sounds") {
            "parse_resource_pack_files::sound"
        } else {
            "parse_resource_pack_files::unknown"
        },
    );
    if name == "pack.mcmeta" {
        let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };
        match PackMcmeta::from_json(json_str) {
            Ok(value) => {
                *pack_mcmeta.lock().unwrap() = Some(value);
            }
            Err(e) => {
                handle_parse_error(logger, assets, name, content, e);
            }
        }
    } else if name.ends_with(".json") {
        match get_type(name) {
            Some("models") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };
                match Model::from_json(overlay, identifier, json_str) {
                    Ok(value) => {
                        assets.push(Asset::Model(value)).unwrap();
                    }
                    Err(e) => {
                        handle_parse_error(logger, assets, name, content, e);
                    }
                }
            }
            Some("blockstates") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };
                match Blockstate::from_json(overlay, identifier, json_str) {
                    Ok(value) => {
                        assets.push(Asset::Blockstate(value)).unwrap();
                    }
                    Err(e) => {
                        handle_parse_error(logger, assets, name, content, e);
                    }
                }
            }
            Some("items") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };

                match Item::from_json(overlay, identifier, json_str) {
                    Ok(value) => {
                        assets.push(Asset::Item(value)).unwrap();
                    }
                    Err(e) => {
                        handle_parse_error(logger, assets, name, content, e);
                    }
                }
            }
            Some("font") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };

                match Font::from_json(overlay, identifier, json_str) {
                    Ok(value) => {
                        assets.push(Asset::Font(value)).unwrap();
                    }
                    Err(e) => {
                        handle_parse_error(logger, assets, name, content, e);
                    }
                }
            }
            Some("atlases") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };

                match AtlasType::from_str(identifier.path.as_str()) {
                    Ok(atlas_type) => match Atlas::from_json(overlay, atlas_type, json_str) {
                        Ok(value) => {
                            assets.push(Asset::Atlas(value)).unwrap();
                        }
                        Err(e) => {
                            handle_parse_error(logger, assets, name, content, e);
                        }
                    },
                    Err(_) => {
                        let _ = logger.send(LogMessage {
                            level: Error,
                            message: format!("Unknown atlas type '{name}'"),
                        });
                        assets.push(Asset::UnknownFile(UnknownFile::new(name.to_owned(), content.to_owned()))).unwrap();
                    }
                }
            }
            Some("lang") => {
                let (overlay, identifier) = parse_path(name);
                let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };

                match Lang::from_json(overlay, identifier, json_str) {
                    Ok(value) => {
                        assets.push(Asset::Lang(value)).unwrap();
                    }
                    Err(e) => {
                        handle_parse_error(logger, assets, name, content, e);
                    }
                }
            }
            _ => {
                if name.ends_with("/sounds.json") {
                    let (overlay, identifier) = parse_path(name);
                    let Some(json_str) = parse_utf8_or_unknown_file(logger, assets, name, content) else { return };
                    match SoundDefinitions::from_json(overlay, identifier.namespace, json_str) {
                        Ok(value) => {
                            assets.push(Asset::SoundDefinitions(value)).unwrap();
                        }
                        Err(e) => {
                            handle_parse_error(logger, assets, name, content, e);
                        }
                    }
                } else {
                    json_file(logger, assets, name, content);
                }
            }
        }
    } else if name.ends_with(".mcmeta") {
        json_file(logger, assets, name, content);
    } else if name.ends_with(".png") {
        if get_type(name) == Some("textures") {
            let (overlay, identifier) = parse_path(name);
            assets.push(Asset::AssetTexture(AssetTexture::new(overlay, identifier, content.to_owned()))).unwrap();
        } else {
            assets.push(Asset::UnknownTexture(UnknownTexture::new(name, content.to_owned()))).unwrap();
        }
    } else if name.ends_with(".vsh") || name.ends_with(".fsh") || name.ends_with(".glsl") {
        assets.push(Asset::Shader(Shader::new(
            name.to_owned(),
            match parse_utf8_or_unknown_file(logger, assets, name, content) {
                Some(value) => value,
                None => return,
            },
        ))).unwrap();
    } else if name.ends_with(".ogg") && get_type(name) == Some("sounds") {
        let (overlay, identifier) = parse_path(name);
        assets.push(Asset::Sound(Sound::new(overlay, identifier, content.to_owned()))).unwrap();
    } else {
        assets.push(Asset::UnknownFile(UnknownFile::new(name.to_owned(), content.to_owned()))).unwrap();
    }
}

fn json_file(logger: &UnboundedSender<LogMessage>, assets: &ArrayQueue<Asset>, name: &str, content: &[u8]) {
    match serde_json::from_slice(content) {
        Ok(value) => {
            let _ = logger.send(
                LogMessage {
                    level: Info,
                    message: format!("JSON file: {}", name)
                }
            );
            assets.push(Asset::Json(Json::new(name.to_owned(), value))).unwrap();
        }
        Err(e) => {
            handle_parse_error(logger, assets, name, content, e);
        }
    }
}

fn parse_utf8_or_unknown_file<'a>(
    logger: &UnboundedSender<LogMessage>,
    assets: &ArrayQueue<Asset>,
    name: &str,
    content: &'a [u8],
) -> Option<&'a str> {
    match std::str::from_utf8(content) {
        Ok(s) => Some(s),
        Err(e) => {
            let _ = logger.send(LogMessage {
                level: Error,
                message: format!("Invalid UTF-8 in '{name}': {e}"),
            });
            assets.push(Asset::UnknownFile(UnknownFile::new(name.to_owned(), content.to_owned()))).unwrap();
            None
        }
    }
}

fn handle_parse_error(
    logger: &UnboundedSender<LogMessage>,
    assets: &ArrayQueue<Asset>,
    name: &str,
    content: &[u8],
    error: impl std::fmt::Display,
) {
    let _ = logger.send(LogMessage {
        level: Error,
        message: format!("Could not parse '{name}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {error}"),
    });
    assets.push(Asset::UnknownFile(UnknownFile::new(name.to_owned(), content.to_owned()))).unwrap();
}
