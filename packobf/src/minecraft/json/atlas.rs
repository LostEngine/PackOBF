use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_atlas_Atlas {
    /// List of sprite sources which can add or remove sprite textures to this atlas.
    pub sources: Vec<java_assets_atlas_SpriteSource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_atlas_FilterPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_atlas_PaletteTexture {
    /// Identifier for the `texture` registry.
    String1(crate::resource_pack::identifier::Identifier),
    /// Identifier for the `texture` registry.
    /// Available since Minecraft 26.3.
    java_assets_atlas_PaletteRef2(crate::resource_pack::identifier::Identifier),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum java_assets_atlas_SpriteSource {
    #[serde(rename = "directory", alias = "minecraft:directory")]
    Directory {
        /// Directory of texture locations to include, relative to the `textures` folder, not including the trailing `/`.
        source: String,
        /// The sprite name prefix, usually ending with `/`.
        prefix: String,
    },
    #[serde(rename = "filter", alias = "minecraft:filter")]
    Filter {
        /// Pattern to remove sprite identifiers already in the atlas. The order of sprite sources is important.
        pattern: java_assets_atlas_FilterPattern,
    },
    #[serde(rename = "paletted_permutations", alias = "minecraft:paletted_permutations")]
    PalettedPermutations {
        /// Identifier for the `texture` registry.
        textures: Vec<crate::resource_pack::identifier::Identifier>,
        palette_key: java_assets_atlas_PaletteTexture,
        permutations: HashMap<String, java_assets_atlas_PaletteTexture>,
        /// Value to use when joining the texture and permutation names to produce the sprite name.
        /// Defaults to `_`.
        #[serde(skip_serializing_if = "Option::is_none")]
        separator: Option<String>,
    },
    #[serde(rename = "single", alias = "minecraft:single")]
    Single {
        /// Identifier for the `texture` registry.
        /// A single texture location of the source.
        resource: crate::resource_pack::identifier::Identifier,
        /// Identifier for the `texture` registry.
        /// The identifier of the sprite that can referenced.
        /// If not specified, matches `resource`.
        #[serde(skip_serializing_if = "Option::is_none")]
        sprite: Option<crate::resource_pack::identifier::Identifier>,
    },
    #[serde(rename = "unstitch", alias = "minecraft:unstitch")]
    Unstitch {
        /// Identifier for the `texture` registry.
        resource: crate::resource_pack::identifier::Identifier,
        /// If set to the resource width, regions will use pixel coordinates.
        #[serde(skip_serializing_if = "Option::is_none")]
        divisor_x: Option<f64>,
        /// If set to the resource height, regions will use pixel coordinates.
        #[serde(skip_serializing_if = "Option::is_none")]
        divisor_y: Option<f64>,
        regions: Vec<java_assets_atlas_UnstitchRegion>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_atlas_UnstitchRegion {
    /// Identifier for the `texture` registry.
    pub sprite: crate::resource_pack::identifier::Identifier,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
