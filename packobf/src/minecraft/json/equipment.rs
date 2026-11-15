use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::*;

#[inline]
pub fn is_none_or_older_than_1_21_11<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_1_21_11(())
}

#[inline]
pub fn is_none_or_older_than_1_21_6<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_1_21_6(())
}

#[inline]
pub fn is_none_or_older_than_26_1<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_26_1(())
}

#[inline]
pub fn is_none_or_older_than_26_3<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_26_3(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsEquipmentTrimOverrideWhenAnonymous {
    /// Identifier for the `trim_pattern` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<crate::resource_pack::identifier::Identifier>,
    /// Identifier for the `trim_material` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<crate::resource_pack::identifier::Identifier>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_Dyeable {
    /// If the item is not dyed, this color is used instead. \
    /// If not specified and the item is not dyed, this layer will be hidden.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_when_undyed: Option<java_util_color_RGB>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_Equipment {
    /// List of layers for each model layer type.
    pub layers: java_assets_equipment_Layers,
    /// Replaces trim texture based on armor trim. \
    /// Only the first entry that matches is applied.
    #[serde(skip_serializing_if = "is_none_or_older_than_26_3")]
    pub trim_overrides: Option<Vec<java_assets_equipment_TrimOverride>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_Layer<T> {
    /// Texture location for this layer, inside `entity/equipment/<layer>/`.
    pub texture: T,
    /// If specified, this layer will be tinted by the color contained in the `dyed_color` component. \
    /// Items not in the `#dyeable` tag are always considered "not dyed".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dyeable: Option<java_assets_equipment_Dyeable>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_Layers {
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humanoid: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub humanoid_leggings: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
    pub humanoid_baby: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wings: Option<Vec<java_assets_equipment_WingsLayer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wolf_body: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horse_body: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llama_body: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_6")]
    pub happy_ghast_body: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub nautilus_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub nautilus_body: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pig_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strider_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub camel_husk_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camel_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horse_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub donkey_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mule_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zombie_horse_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
    /// Identifier for the `texture` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeleton_horse_saddle: Option<Vec<java_assets_equipment_Layer<crate::resource_pack::identifier::TextureId>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_TrimOverride {
    pub when: JavaAssetsEquipmentTrimOverrideWhenAnonymous,
    /// Identifier.
    /// When present, overrides the base texture provided by trim pattern. \
    /// The texture is located under `trims/entity/<layer>/`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texture: Option<crate::resource_pack::identifier::Identifier>,
    /// Identifier for the `texture` registry.
    /// When present, overrides the palette texture provided by trim material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<crate::resource_pack::identifier::TextureId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_equipment_WingsLayer<T> {
    /// Texture location for this layer, inside `entity/equipment/<layer>/`.
    pub texture: T,
    /// If specified, this layer will be tinted by the color contained in the `dyed_color` component. \
    /// Items not in the `#dyeable` tag are always considered "not dyed".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dyeable: Option<java_assets_equipment_Dyeable>,
    /// Whether this layer texture should be overridden by the player's custom elytra texture. \
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_player_texture: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_util_color_RGB {
    Integer1(i32),
    List22([f32; 3]),
}
