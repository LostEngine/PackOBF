use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::*;

#[inline]
pub fn is_none_or_newer_than_1_21_2<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_newer_than_1_21_2(())
}

#[inline]
pub fn is_none_or_newer_than_26_2<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_newer_than_26_2(())
}

#[inline]
pub fn is_none_or_older_than_1_21_2<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_1_21_2(())
}

#[inline]
pub fn is_none_or_older_than_26_3<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_26_3(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsModelModelDisplayAnonymous {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 3]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsModelModelElementFacesAnonymous {
    pub texture: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv: Option<[f32; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cullface: Option<java_util_direction_Direction>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub tintindex: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsModelModelElementRotationObject11Anonymous {
    pub origin: [f32; 3],
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rescale: Option<bool>,
    pub axis: java_util_direction_Axis,
    pub angle: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsModelModelElementRotationObject22Anonymous {
    pub origin: [f32; 3],
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rescale: Option<bool>,
    #[serde(flatten)]
    pub entries_1: HashMap<java_util_direction_Axis, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsModelModelOverridesAnonymous {
    pub predicate: HashMap<java_assets_model_Predicates, f32>,
    /// Identifier for the `model` registry.
    pub model: crate::resource_pack::identifier::ModelId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaAssetsModelModelTextures {
    String1(crate::resource_pack::identifier::TextureId),
    /// Available since Minecraft 26.1.
    java_assets_model_TextureMaterial2(java_assets_model_TextureMaterial),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_model_CustomizableItemDisplayContext {
    #[serde(rename = "firstperson_righthand", alias = "minecraft:firstperson_righthand")]
    FirstPersonRighthand,
    #[serde(rename = "firstperson_lefthand", alias = "minecraft:firstperson_lefthand")]
    FirstPersonLefthand,
    #[serde(rename = "thirdperson_righthand", alias = "minecraft:thirdperson_righthand")]
    ThirdPersonRighthand,
    #[serde(rename = "thirdperson_lefthand", alias = "minecraft:thirdperson_lefthand")]
    ThirdPersonLefthand,
    #[serde(rename = "gui", alias = "minecraft:gui")]
    Gui,
    #[serde(rename = "head", alias = "minecraft:head")]
    Head,
    #[serde(rename = "ground", alias = "minecraft:ground")]
    Ground,
    #[serde(rename = "fixed", alias = "minecraft:fixed")]
    Fixed,
    /// Available since Minecraft 1.21.9.
    #[serde(rename = "on_shelf", alias = "minecraft:on_shelf")]
    OnShelf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_model_Model {
    /// Identifier for the `model` registry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<crate::resource_pack::identifier::ModelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambientocclusion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gui_light: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub textures: Option<HashMap<String, JavaAssetsModelModelTextures>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<java_assets_model_ModelElement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<HashMap<java_assets_model_CustomizableItemDisplayContext, JavaAssetsModelModelDisplayAnonymous>>,
    #[serde(skip_serializing_if = "is_none_or_newer_than_1_21_2")]
    pub overrides: Option<Vec<JavaAssetsModelModelOverridesAnonymous>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_model_ModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub faces: HashMap<java_util_direction_Direction, JavaAssetsModelModelElementFacesAnonymous>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<java_assets_model_ModelElementRotation>,
    #[serde(skip_serializing_if = "is_none_or_newer_than_26_2")]
    pub shade: Option<bool>,
    #[serde(skip_serializing_if = "is_none_or_older_than_26_3")]
    pub shade_direction_override: Option<java_util_direction_Direction>,
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_2")]
    pub light_emission: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_model_ModelElementRotation {
    Object11(JavaAssetsModelModelElementRotationObject11Anonymous),
    Object22(JavaAssetsModelModelElementRotationObject22Anonymous),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_model_Predicates {
    /// Used on compasses to determine the current angle, expressed in a decimal value of less than one.
    #[serde(rename = "angle", alias = "minecraft:angle")]
    Angle,
    /// Used on shields to determine if currently blocking. If `1`, the player is blocking.
    #[serde(rename = "blocking", alias = "minecraft:blocking")]
    Blocking,
    /// Used on Elytra to determine if broken. If `1`, the Elytra is broken.
    #[serde(rename = "broken", alias = "minecraft:broken")]
    Broken,
    /// Used on fishing rods to determine if the fishing rod has been cast. If `1`, the fishing rod has been cast.
    #[serde(rename = "cast", alias = "minecraft:cast")]
    Cast,
    /// Used on crossbows to determine if they are charged with any projectile. If `1`, the crossbow is charged.
    #[serde(rename = "charged", alias = "minecraft:charged")]
    Charged,
    /// Used on ender pearls and chorus fruit to determine the remaining cooldown, expressed in a decimal value between `0` and `1`.
    #[serde(rename = "cooldown", alias = "minecraft:cooldown")]
    Cooldown,
    /// Used on any item and is compared to the CustomModelData NBT, expressed in an integer value.
    ///
    /// The number is still internally converted to float, causing a precision loss for some numbers above 16 million. \
    /// If the value read from the item data is greater than or equal to the value used for the predicate, the predicate is positive.
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData,
    /// Used on items with durability to determine the amount of damage, expressed in a decimal value between `0` and `1`.
    #[serde(rename = "damage", alias = "minecraft:damage")]
    Damage,
    /// Used on items with durability to determine if it is damaged. If 1, the item is damaged. \
    /// Note that if an item has the unbreakable tag, this may be `0` while the item has a non-zero `'damage'` tag.
    #[serde(rename = "damaged", alias = "minecraft:damaged")]
    Damaged,
    /// Used on crossbows. If `1`, the crossbow is charged with a firework rocket.
    #[serde(rename = "firework", alias = "minecraft:firework")]
    Firework,
    /// Used for honey level on bee nests and beehives.
    #[serde(rename = "honey_level", alias = "minecraft:honey_level")]
    HoneyLevel,
    /// Determines the model used by left handed players. It affects the item they see in inventories, along with the item players see them holding or wearing.
    #[serde(rename = "lefthanded", alias = "minecraft:lefthanded")]
    Lefthanded,
    /// Used for the `level` block state on light items.
    #[serde(rename = "level", alias = "minecraft:level")]
    Level,
    /// Determines the amount a bow or crossbow has been pulled, expressed in a decimal value of less than one.
    #[serde(rename = "pull", alias = "minecraft:pull")]
    Pull,
    /// Used on bows and crossbows to determine if the bow is being pulled. If `1`, the bow is currently being pulled.
    #[serde(rename = "pulling", alias = "minecraft:pulling")]
    Pulling,
    /// Used on the trident to determine if the trident is ready to be thrown by the player. If `1`, the trident is ready for fire.
    #[serde(rename = "throwing", alias = "minecraft:throwing")]
    Throwing,
    /// Used on clocks to determine the current time, expressed in a decimal value of less than one.
    #[serde(rename = "time", alias = "minecraft:time")]
    Time,
    /// Whether the goat horn is being used.
    #[serde(rename = "tooting", alias = "minecraft:tooting")]
    Tooting,
    #[serde(rename = "trim_type", alias = "minecraft:trim_type")]
    TrimType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_model_TextureMaterial {
    /// Identifier for the `texture` registry.
    pub sprite: crate::resource_pack::identifier::TextureId,
    /// Whether the texture should be forced into the translucent render pass. \
    /// Textures without any translucent pixels are not assigned to the translucent pass by default. \
    /// Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_translucent: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_util_direction_Axis {
    #[serde(rename = "x", alias = "minecraft:x")]
    X,
    #[serde(rename = "y", alias = "minecraft:y")]
    Y,
    #[serde(rename = "z", alias = "minecraft:z")]
    Z,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_util_direction_Direction {
    #[serde(rename = "down", alias = "minecraft:down")]
    Down,
    #[serde(rename = "up", alias = "minecraft:up")]
    Up,
    #[serde(rename = "north", alias = "minecraft:north")]
    North,
    #[serde(rename = "east", alias = "minecraft:east")]
    East,
    #[serde(rename = "south", alias = "minecraft:south")]
    South,
    #[serde(rename = "west", alias = "minecraft:west")]
    West,
}
