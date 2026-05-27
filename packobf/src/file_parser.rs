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
use crate::resource_pack::resource_pack::ResourcePack;
use crate::LogLevel::Error;
use crate::{LogMessage, Progress};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Sender;

pub fn parse_resource_pack_files(
    logger: &UnboundedSender<LogMessage>,
    entries: &mut Vec<(String, Vec<u8>)>,
    progress: Sender<Progress>,
    pack: Arc<ResourcePack>,
) {
    entries.par_iter_mut().for_each(move |(name, content)| {
        parse_resource_pack_file(logger, &progress, &pack, name, content);
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
    let _ = crate::profiler::profiler::ScopeTimer::new(
        if name.ends_with(".json") || name.ends_with(".mcmeta") {
            "parse_resource_pack_files::json"
        } else if name.ends_with(".png") && crate::get_type(&name) == Some("textures") {
            "parse_resource_pack_files::texture"
        } else if name.ends_with(".vsh") || name.ends_with(".fsh") || name.ends_with(".glsl") {
            "parse_resource_pack_files::shader"
        } else if name.ends_with(".ogg") && crate::get_type(&name) == Some("sounds") {
            "parse_resource_pack_files::sound"
        } else {
            "parse_resource_pack_files::unknown"
        },
    );
    if name.ends_with(".json") {
        let asset_type = crate::get_type(&name);
        if asset_type == Some("models")
            || asset_type == Some("blockstates")
            || asset_type == Some("items")
            || asset_type == Some("font")
            || asset_type == Some("atlases")
        {
            let (overlay, identifier) = crate::parse_path(&name);
            let json_str = String::from_utf8(content.to_owned()).unwrap();
            match asset_type.unwrap() {
                "models" => match Model::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.model(value);
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                                level: Error,
                                message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                            });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                },
                "blockstates" => match Blockstate::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.blockstate(value);
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                                level: Error,
                                message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                            });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                },
                "items" => match Item::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.item(value);
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                                level: Error,
                                message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                            });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                },
                "font" => match Font::from_json(overlay, identifier, &json_str) {
                    Ok(value) => {
                        pack.font(value);
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                                level: Error,
                                message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                            });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                },
                "atlases" => match AtlasType::from_str(identifier.path.as_str()) {
                    Ok(atlas_type) => match Atlas::from_json(overlay, atlas_type, &json_str) {
                        Ok(value) => {
                            pack.atlas(value);
                        }
                        Err(e) => {
                            let _ = logger.send(LogMessage {
                                        level: Error,
                                        message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                                    });
                            pack.unknown_file(ResourcePackFile::new(
                                name.to_owned(),
                                content.to_owned(),
                            ));
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
                },
                _ => unreachable!(),
            }
        } else {
            if name.ends_with("/sounds.json") {
                let (overlay, identifier) = crate::parse_path(&name);
                let json_str = String::from_utf8(content.to_owned()).unwrap();
                match SoundDefinitions::from_json(overlay, identifier.namespace, &json_str) {
                    Ok(value) => {
                        pack.sound_definitions(value);
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                            level: Error,
                            message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                        });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                }
            } else {
                match serde_json::from_slice(&content) {
                    Ok(value) => {
                        pack.json_file(Json::new(name.to_owned(), value));
                    }
                    Err(e) => {
                        let _ = logger.send(LogMessage {
                            level: Error,
                            message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                        });
                        pack.unknown_file(ResourcePackFile::new(
                            name.to_owned(),
                            content.to_owned(),
                        ));
                    }
                }
            }
        }
    } else if name.ends_with(".mcmeta") {
        match serde_json::from_slice(&content) {
            Ok(value) => {
                pack.json_file(Json::new(name.to_owned(), value));
            }
            Err(e) => {
                let _ = logger.send(LogMessage {
                    level: Error,
                    message: format!("Could not parse '{}'. This is most likely not a packobf issue but a json file that is malformed. Treating it as an unknown file. Error: {}", name, e),
                });
                pack.unknown_file(ResourcePackFile::new(name.to_owned(), content.to_owned()));
            }
        }
    } else if name.ends_with(".png") && crate::get_type(&name) == Some("textures") {
        let (overlay, identifier) = crate::parse_path(&name);
        pack.texture(Texture::new(overlay, identifier, content.to_owned()));
    } else if name.ends_with(".vsh") || name.ends_with(".fsh") || name.ends_with(".glsl") {
        pack.shader(Shader::new(
            name.to_owned(),
            String::from_utf8(content.to_owned()).unwrap(),
        ));
    } else if name.ends_with(".ogg") && crate::get_type(&name) == Some("sounds") {
        let (overlay, identifier) = crate::parse_path(&name);
        pack.sound(Sound::new(overlay, identifier, content.to_owned()));
    } else {
        pack.unknown_file(ResourcePackFile::new(name.to_owned(), content.to_owned()));
    }
}
