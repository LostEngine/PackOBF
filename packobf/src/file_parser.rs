use crate::minecraft::builtin_files::AtlasType;
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
use crate::resource_pack::pack::ResourcePack;
use crate::LogLevel::Error;
use crate::{get_type, parse_path, LogMessage, Progress};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::str::FromStr;
use std::sync::Arc;
use rayon::ThreadPool;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Sender;
use crate::resource_pack::files::unknowntexture::UnknownTexture;

pub fn parse_resource_pack_files(
    logger: &UnboundedSender<LogMessage>,
    entries: &mut Vec<(String, Vec<u8>)>,
    progress: Sender<Progress>,
    pack: Arc<ResourcePack>,
    thread_pool: &ThreadPool
) {
    thread_pool.install(|| {
        entries.par_iter_mut().for_each(move |(name, content)| {
            parse_resource_pack_file(logger, &progress, &pack, name, content);
        });
    });
}

fn parse_resource_pack_file(
    logger: &UnboundedSender<LogMessage>,
    progress: &Sender<Progress>,
    pack: &Arc<ResourcePack>,
    name: &mut String,
    content: &mut Vec<u8>,
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
    if name.ends_with(".json") {
        match get_type(name) {
            Some("models") => {
                let (overlay, identifier) = parse_path(name);
                let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                    Some(value) => value,
                    None => return,
                };
                match Model::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.model(value);
                    }
                    Err(e) => {
                        handle_parse_error(logger, pack, name, content, e);
                    }
                }
            }
            Some("blockstates") => {
                let (overlay, identifier) = parse_path(name);
                let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                    Some(value) => value,
                    None => return,
                };
                match Blockstate::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.blockstate(value);
                    }
                    Err(e) => {
                        handle_parse_error(logger, pack, name, content, e);
                    }
                }
            }
            Some("items") => {
                let (overlay, identifier) = parse_path(name);
                let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                    Some(value) => value,
                    None => return,
                };

                match Item::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.item(value);
                    }
                    Err(e) => {
                        handle_parse_error(logger, pack, name, content, e);
                    }
                }
            }
            Some("font") => {
                let (overlay, identifier) = parse_path(name);
                let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                    Some(value) => value,
                    None => return,
                };

                match Font::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.font(value);
                    }
                    Err(e) => {
                        handle_parse_error(logger, pack, name, content, e);
                    }
                }
            }
            Some("atlases") => {
                let (overlay, identifier) = parse_path(name);
                let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                    Some(value) => value,
                    None => return,
                };

                match AtlasType::from_str(identifier.path.as_str()) {
                    Ok(atlas_type) => match Atlas::from_json(overlay, atlas_type, &json_str) {
                        Ok(value) => {
                            pack.atlas(value);
                        }
                        Err(e) => {
                            handle_parse_error(logger, pack, name, content, e);
                        }
                    },
                    Err(_) => {
                        let _ = logger.send(LogMessage {
                            level: Error,
                            message: format!("Unknown atlas type '{}'", name,),
                        });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                }
            }
            _ => {
                if name.ends_with("/sounds.json") {
                    let (overlay, identifier) = parse_path(name);
                    let json_str = match parse_utf8_or_unknown_file(logger, pack, name, content) {
                        Some(value) => value,
                        None => return,
                    };
                    match SoundDefinitions::from_json(overlay, identifier.namespace, &json_str) {
                        Ok(value) => {
                            pack.sound_definitions(value);
                        }
                        Err(e) => {
                            handle_parse_error(logger, pack, name, content, e);
                        }
                    }
                } else {
                    json_file(logger, pack, name, content);
                }
            }
        }
    } else if name.ends_with(".mcmeta") {
        json_file(logger, pack, name, content);
    } else if name.ends_with(".png") {
        if get_type(name) == Some("textures") {
            let (overlay, identifier) = parse_path(name);
            pack.texture(Texture::new(overlay, identifier, content.to_owned()));
        } else {
            pack.unknown_texture(UnknownTexture::new(name.as_str(), content.to_owned()))
        }
    } else if name.ends_with(".vsh") || name.ends_with(".fsh") || name.ends_with(".glsl") {
        pack.shader(Shader::new(
            name.to_owned(),
            match parse_utf8_or_unknown_file(logger, pack, name, content) {
                Some(value) => value,
                None => return,
            },
        ));
    } else if name.ends_with(".ogg") && get_type(name) == Some("sounds") {
        let (overlay, identifier) = parse_path(name);
        pack.sound(Sound::new(overlay, identifier, content.to_owned()));
    } else {
        pack.unknown_file(ResourcePackFile::new(name.to_owned(), content.to_owned()));
    }
}

fn json_file(logger: &UnboundedSender<LogMessage>, pack: &Arc<ResourcePack>, name: &mut String, content: &mut Vec<u8>) {
    match serde_json::from_slice(content) {
        Ok(value) => {
            pack.json_file(Json::new(name.to_owned(), value));
        }
        Err(e) => {
            handle_parse_error(logger, pack, name, content, e);
        }
    }
}

fn parse_utf8_or_unknown_file<'a>(
    logger: &UnboundedSender<LogMessage>,
    pack: &Arc<ResourcePack>,
    name: &str,
    content: &'a [u8],
) -> Option<&'a str> {
    match std::str::from_utf8(content) {
        Ok(s) => Some(s),
        Err(e) => {
            let _ = logger.send(LogMessage {
                level: Error,
                message: format!("Invalid UTF-8 in '{}': {}", name, e),
            });
            pack.unknown_file(ResourcePackFile::new(name.to_owned(), content.to_owned()));
            None
        }
    }
}

fn handle_parse_error(
    logger: &UnboundedSender<LogMessage>,
    pack: &Arc<ResourcePack>,
    name: &str,
    content: &[u8],
    error: impl std::fmt::Display,
) {
    let _ = logger.send(LogMessage {
        level: Error,
        message: format!(
            "Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}",
            name, error
        ),
    });
    pack.unknown_file(ResourcePackFile::new(name.to_owned(), content.to_owned()));
}
