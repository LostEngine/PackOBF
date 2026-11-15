use crate::minecraft::builtin_files::AtlasType;
use crate::minecraft::json::atlas::*;
use crate::resource_pack::identifier::Identifier;
use crate::utils::clean_json_numbers;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atlas {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub atlas_type: AtlasType,

    #[serde(flatten)]
    pub json: java_assets_atlas_Atlas,
}

impl std::fmt::Display for Atlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(f, "{}", serde_json::to_string(&val).map_err(|_| std::fmt::Error)?)
    }
}

impl Atlas {
    pub fn from_json(
        overlay: impl Into<String>,
        atlas_type: AtlasType,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut atlas: Self = serde_json::from_str(json)?;

        atlas.overlay = overlay.into();
        atlas.atlas_type = atlas_type;

        Ok(atlas)
    }

    pub fn path(&self) -> String {
        let prefix = match self.overlay.as_str() {
            "" => "".to_string(),
            x => format!("{x}/"),
        };
        format!(
            "{prefix}assets/minecraft/atlases/{}.json",
            self.atlas_type
        )
    }


    pub fn get_identifier(&self, texture_id: &Identifier) -> Option<Identifier> {
        let mut result: Option<Identifier> = None;

        for source in self.json.sources.iter() {
            match source {
                java_assets_atlas_SpriteSource::Single { resource, sprite } => {
                    let name = sprite.as_ref().unwrap_or(resource);
                    if name.namespace == texture_id.namespace && name.path == texture_id.path {
                        result = Some(resource.clone());
                        break;
                    }
                }
                java_assets_atlas_SpriteSource::Directory { source, prefix }
                    if texture_id.path.starts_with(prefix) => {
                        let remaining_path = &texture_id.path[prefix.len()..];
                        let resource_path = format!("{source}{remaining_path}");

                        result = Some(Identifier {
                            namespace: texture_id.namespace.clone(),
                            path: resource_path,
                        });
                        break;
                    }
                _ => {}
            }
        }

        if result.is_some() {
            for source in self.json.sources.iter() {
                if let java_assets_atlas_SpriteSource::Filter { pattern, .. } = source {
                    let ns_match = pattern.namespace.as_ref().is_none_or(|ns_regex| Regex::new(ns_regex)
                            .is_ok_and(|re| re.is_match(&texture_id.namespace)));

                    let path_match = pattern.path.as_ref().is_none_or(|path_regex| Regex::new(path_regex)
                            .is_ok_and(|re| re.is_match(&texture_id.path)));

                    if ns_match && path_match {
                        return None;
                    }
                }
            }
        }

        result
    }

    pub fn get_sprites(&self, texture_id: &Identifier) -> Vec<Identifier> {
        let mut sprites = Vec::new();
        for source in &self.json.sources {
            match source {
                java_assets_atlas_SpriteSource::Single { resource, sprite }
                    if resource == texture_id =>
                {
                    sprites.push(sprite.clone().unwrap_or_else(|| resource.clone()));
                }
                java_assets_atlas_SpriteSource::Directory { source, prefix } => {
                    if let Some(path) = texture_id.path.strip_prefix(source) {
                        sprites.push(Identifier::new(
                            texture_id.namespace.clone(),
                            format!("{prefix}{path}"),
                        ));
                    }
                }
                java_assets_atlas_SpriteSource::Unstitch { resource, regions, .. } if resource == texture_id => {
                    sprites.extend(regions.iter().map(|region| region.sprite.clone()));
                }
                java_assets_atlas_SpriteSource::PalettedPermutations {
                    textures,
                    permutations,
                    separator,
                    ..
                } if textures.contains(texture_id) => {
                    sprites.extend(permutations.keys().map(|permutation| {
                        Identifier::new(
                            texture_id.namespace.clone(),
                            format!("{}{}{permutation}", texture_id.path, separator.clone().unwrap_or("".into())),
                        )
                    }));
                }
                _ => {}
            }
        }

        sprites.retain(|sprite| {
            !self.json.sources.iter().any(|source| match source {
                java_assets_atlas_SpriteSource::Filter { pattern } => {
                    let ns_match = pattern.namespace.as_ref().is_none_or(|regex| {
                        Regex::new(regex).is_ok_and(|re| re.is_match(&sprite.namespace))
                    });
                    let path_match = pattern.path.as_ref().is_none_or(|regex| {
                        Regex::new(regex).is_ok_and(|re| re.is_match(&sprite.path))
                    });
                    ns_match && path_match
                }
                _ => false,
            })
        });
        sprites
    }

    pub fn has_declared_sprite(&self, sprite_id: &Identifier) -> bool {
        self.json.sources.iter().any(|source| match source {
            java_assets_atlas_SpriteSource::Single { resource, sprite } => {
                sprite.as_ref().unwrap_or(resource) == sprite_id
            }
            java_assets_atlas_SpriteSource::Unstitch { regions, .. } => {
                regions.iter().any(|region| &region.sprite == sprite_id)
            }
            java_assets_atlas_SpriteSource::PalettedPermutations { textures, permutations, separator, .. } => {
                textures.iter().any(|texture| {
                    permutations.keys().any(|permutation| {
                        Identifier::new(
                            texture.namespace.clone(),
                            format!("{}{}{permutation}", texture.path, separator.clone().unwrap_or("".into())),
                        ) == *sprite_id
                    })
                })
            }
            java_assets_atlas_SpriteSource::Directory { .. } | java_assets_atlas_SpriteSource::Filter { .. } => false,
        }) && !self.json.sources.iter().any(|source| match source {
            java_assets_atlas_SpriteSource::Filter { pattern } => {
                let ns_match = pattern.namespace.as_ref().is_none_or(|regex| {
                    Regex::new(regex).is_ok_and(|re| re.is_match(&sprite_id.namespace))
                });
                let path_match = pattern.path.as_ref().is_none_or(|regex| {
                    Regex::new(regex).is_ok_and(|re| re.is_match(&sprite_id.path))
                });
                ns_match && path_match
            }
            _ => false,
        })
    }
}
