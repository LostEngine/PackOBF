use crate::resource_pack::identifier::Identifier;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::minecraft::builtin_files::AtlasType;
use crate::utils::clean_json_numbers;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atlas {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub atlas_type: AtlasType,

    pub sources: Vec<Source>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "directory", alias = "minecraft:directory")]
    Directory { source: String, prefix: String },
    #[serde(rename = "single", alias = "minecraft:single")]
    Single {
        resource: Identifier,
        sprite: Option<Identifier>,
    },
    #[serde(rename = "filter", alias = "minecraft:filter")]
    Filter {
        pattern: FilterPattern,
    },
    #[serde(rename = "unstitch", alias = "minecraft:unstitch")]
    Unstitch {
        resource: Identifier,
        #[serde(default = "crate::json::default_one", skip_serializing_if = "crate::json::is_one")]
        divisor_x: f32,
        #[serde(default = "crate::json::default_one", skip_serializing_if = "crate::json::is_one")]
        divisor_y: f32,
        regions: Vec<UnstitchRegion>,
    },
    #[serde(
        rename = "paletted_permutations",
        alias = "minecraft:paletted_permutations"
    )]
    PalettedPermutations {
        textures: Vec<Identifier>,
        palette_key: String,
        permutations: HashMap<String, Identifier>,
        #[serde(default = "crate::json::default_underscore", skip_serializing_if = "crate::json::is_underscore")]
        separator: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnstitchRegion {
    sprite: Identifier,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
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

        for source in self.sources.iter() {
            match source {
                Source::Single { resource, sprite } => {
                    let name = sprite.as_ref().unwrap_or(resource);
                    if name.namespace == texture_id.namespace && name.path == texture_id.path {
                        result = Some(resource.clone());
                        break;
                    }
                }
                Source::Directory { source, prefix }
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
            for source in self.sources.iter() {
                if let Source::Filter { pattern, .. } = source {
                    let ns_match = pattern.namespace.as_ref().is_none_or( |ns_regex| Regex::new(ns_regex)
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
}
