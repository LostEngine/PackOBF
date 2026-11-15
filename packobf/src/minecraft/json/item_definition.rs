use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::*;

#[inline]
pub fn is_none_or_newer_than_1_21_11<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_newer_than_1_21_11(())
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsItemDefinitionItemModelRangeDispatchEntriesAnonymous {
    pub threshold: f32,
    pub model: java_assets_item_definition_ItemModel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsItemDefinitionItemModelSpecialModelAnonymous {
    #[serde(flatten)]
    pub r#type: java_assets_item_definition_SpecialModelType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaAssetsItemDefinitionSelectCaseWhen<T> {
    java_assets_item_definition_T1(T),
    List22(Vec<T>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaAssetsItemDefinitionSelectPropertyTypeBlockStateBlockStateProperty {
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaWorldEntityDisplayTransformationObject11Anonymous {
    /// Translation in [x, y, z].
    pub translation: [f32; 3],
    /// Using this rotation is enough for most transformations.
    pub left_rotation: java_world_entity_display_Rotation,
    /// For more complex transformations. Applied **before** scaling.
    pub right_rotation: java_world_entity_display_Rotation,
    /// Scale in [x, y, z].
    pub scale: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaWorldEntityDisplayTransformationVariant22 {
    List11([f32; 16]),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_item_definition_ActuallyTranslucentRGB {
    Integer1(i32),
    List22([f32; 3]),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_BannerAttachment {
    #[serde(rename = "wall", alias = "minecraft:wall")]
    Wall,
    #[serde(rename = "ground", alias = "minecraft:ground")]
    Ground,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_BedPart {
    #[serde(rename = "head", alias = "minecraft:head")]
    Head,
    #[serde(rename = "foot", alias = "minecraft:foot")]
    Foot,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_ChestType {
    #[serde(rename = "single", alias = "minecraft:single")]
    Single,
    #[serde(rename = "left", alias = "minecraft:left")]
    Left,
    #[serde(rename = "right", alias = "minecraft:right")]
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_CompassTarget {
    /// Always an invalid target.
    #[serde(rename = "none", alias = "minecraft:none")]
    None,
    /// Points at world spawn.
    #[serde(rename = "spawn", alias = "minecraft:spawn")]
    Spawn,
    /// Points at the location stored in the `lodestone_tracker` component.
    #[serde(rename = "lodestone", alias = "minecraft:lodestone")]
    Lodestone,
    /// Points at the last player death location.
    #[serde(rename = "recovery", alias = "minecraft:recovery")]
    Recovery,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "property")]
pub enum java_assets_item_definition_ConditionalPropertyType {
    #[serde(rename = "component", alias = "minecraft:component")]
    Component {
        /// The component predicate to check.
        predicate: String,
        /// The predicate-specific value.
        value: serde_json::Value,
    },
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData {
        /// The index of the `flags` list in the `custom_model_data` component. Defaults to 0.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        index: Option<i32>,
    },
    #[serde(rename = "has_component", alias = "minecraft:has_component")]
    HasComponent {
        /// Identifier for the `data_component_type` registry.
        component: crate::resource_pack::identifier::Identifier,
        /// Whether the default components should be handled as "no component". Defaults to false.
        #[serde(skip_serializing_if = "Option::is_none")]
        ignore_default: Option<bool>,
    },
    #[serde(rename = "keybind_down", alias = "minecraft:keybind_down")]
    KeybindDown {
        /// The keybind ID to check for.
        keybind: java_util_text_Keybind,
    },
    #[serde(rename = "view_entity", alias = "minecraft:view_entity")]
    ViewEntity {},
    #[serde(rename = "broken", alias = "minecraft:broken")]
    Broken {},
    #[serde(rename = "bundle/has_selected_item", alias = "minecraft:bundle/has_selected_item")]
    BundleHasSelectedItem {},
    #[serde(rename = "carried", alias = "minecraft:carried")]
    Carried {},
    #[serde(rename = "damaged", alias = "minecraft:damaged")]
    Damaged {},
    #[serde(rename = "extended_view", alias = "minecraft:extended_view")]
    ExtendedView {},
    #[serde(rename = "fishing_rod/cast", alias = "minecraft:fishing_rod/cast")]
    FishingRod {},
    #[serde(rename = "selected", alias = "minecraft:selected")]
    Selected {},
    #[serde(rename = "using_item", alias = "minecraft:using_item")]
    UsingItem {},
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_CopperGolemStatuePose {
    #[serde(rename = "standing", alias = "minecraft:standing")]
    Standing,
    #[serde(rename = "sitting", alias = "minecraft:sitting")]
    Sitting,
    #[serde(rename = "running", alias = "minecraft:running")]
    Running,
    #[serde(rename = "star", alias = "minecraft:star")]
    Star,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_CrossbowChargeType {
    #[serde(rename = "none", alias = "minecraft:none")]
    NONE,
    #[serde(rename = "arrow", alias = "minecraft:arrow")]
    ARROW,
    #[serde(rename = "rocket", alias = "minecraft:rocket")]
    ROCKET,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_EndCubeEffectType {
    #[serde(rename = "portal", alias = "minecraft:portal")]
    Portal,
    #[serde(rename = "gateway", alias = "minecraft:gateway")]
    Gateway,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_HangingSignAttachment {
    #[serde(rename = "wall", alias = "minecraft:wall")]
    Wall,
    #[serde(rename = "ceiling", alias = "minecraft:ceiling")]
    Ceiling,
    #[serde(rename = "ceiling_middle", alias = "minecraft:ceiling_middle")]
    CeilingMiddle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_HeadType {
    #[serde(rename = "creeper", alias = "minecraft:creeper")]
    Creeper,
    #[serde(rename = "dragon", alias = "minecraft:dragon")]
    Dragon,
    #[serde(rename = "piglin", alias = "minecraft:piglin")]
    Piglin,
    #[serde(rename = "player", alias = "minecraft:player")]
    Player,
    #[serde(rename = "skeleton", alias = "minecraft:skeleton")]
    Skeleton,
    #[serde(rename = "wither_skeleton", alias = "minecraft:wither_skeleton")]
    WitherSkeleton,
    #[serde(rename = "zombie", alias = "minecraft:zombie")]
    Zombie,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_item_definition_ItemDefinition {
    pub model: java_assets_item_definition_ItemModel,
    /// Whether the down-and-up animation should be played in first-person view when the item stack is changed.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hand_animation_on_swap: Option<bool>,
    /// Whether the item model is allowed to be bigger than its item slot.
    /// Defaults to `false`, which clips the item model in GUI to the item slot size.
    /// The behavior of `true` is **not** officially supported.
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_6")]
    pub oversized_in_gui: Option<bool>,
    /// How fast the item moves up and down when swapping items in hotbar.
    /// Defaults to 1.0
    #[serde(skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub swap_animation_scale: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum java_assets_item_definition_ItemModel {
    #[serde(rename = "bundle/selected_item", alias = "minecraft:bundle/selected_item")]
    BundleSelectedItem {},
    #[serde(rename = "composite", alias = "minecraft:composite")]
    Composite {
        models: Vec<Self>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "condition", alias = "minecraft:condition")]
    Condition {
        #[serde(flatten)]
        property: java_assets_item_definition_ConditionalPropertyType,
        on_true: Box<Self>,
        on_false: Box<Self>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "model", alias = "minecraft:model")]
    Model {
        /// Identifier for the `model` registry.
        model: crate::resource_pack::identifier::ModelId,
        #[serde(skip_serializing_if = "Option::is_none")]
        tints: Option<Vec<java_assets_item_definition_ModelTint>>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "range_dispatch", alias = "minecraft:range_dispatch")]
    RangeDispatch {
        #[serde(flatten)]
        property: java_assets_item_definition_NumericPropertyType,
        /// Factor to multiply the property value with. Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        scale: Option<f32>,
        /// List of ranges. Will select last entry with threshold less or equal to value.
        /// Order does not matter, list will be sorted by threshold in ascending order.
        entries: Vec<JavaAssetsItemDefinitionItemModelRangeDispatchEntriesAnonymous>,
        /// Item model to render if no entries were less or equal to the value.
        #[serde(skip_serializing_if = "Option::is_none")]
        fallback: Option<Box<Self>>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "select", alias = "minecraft:select")]
    Select {
        #[serde(flatten)]
        property: java_assets_item_definition_SelectPropertyType,
        /// Item model to render if none of the cases matched the value.
        #[serde(skip_serializing_if = "Option::is_none")]
        fallback: Option<Box<Self>>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "special", alias = "minecraft:special")]
    Special {
        /// Renders a special hardcoded model.
        model: JavaAssetsItemDefinitionItemModelSpecialModelAnonymous,
        /// Identifier for the `model` registry.
        /// Base model, providing transformations, particle texture and GUI light.
        base: crate::resource_pack::identifier::ModelId,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        transformation: Option<java_world_entity_display_Transformation>,
    },
    #[serde(rename = "empty", alias = "minecraft:empty")]
    Empty {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum java_assets_item_definition_ModelTint {
    #[serde(rename = "constant", alias = "minecraft:constant")]
    Constant {
        /// Constant tint color to apply.
        value: java_util_color_RGB,
    },
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData {
        /// The index of the `colors` list in the `custom_model_data` component. Defaults to 0.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        index: Option<i32>,
        /// Tint to apply when the `custom_model_data` component is not present, or when it doesn't have a color in the specified index.
        default: java_util_color_RGB,
    },
    #[serde(rename = "dye", alias = "minecraft:dye")]
    Dye {
        /// Tint to apply when the `dyed_color` component is not present.
        default: java_assets_item_definition_ActuallyTranslucentRGB,
    },
    #[serde(rename = "firework", alias = "minecraft:firework")]
    Firework {
        /// Tint to apply when the `firework_explosion` component is not present.
        default: java_assets_item_definition_ActuallyTranslucentRGB,
    },
    #[serde(rename = "grass", alias = "minecraft:grass")]
    Grass {
        temperature: f32,
        downfall: f32,
    },
    #[serde(rename = "map_color", alias = "minecraft:map_color")]
    MapColor {
        /// Tint to apply when the `map_color` component is not present.
        default: java_util_color_RGB,
    },
    #[serde(rename = "potion", alias = "minecraft:potion")]
    Potion {
        /// Tint to apply when the `potion_contents` component is not present, or it has no effects and no `custom_color` is set.
        default: java_util_color_RGB,
    },
    #[serde(rename = "team", alias = "minecraft:team")]
    Team {
        /// Tint to apply when there is no context entity, entity is not in a team or the team has no color.
        default: java_util_color_RGB,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "property")]
pub enum java_assets_item_definition_NumericPropertyType {
    #[serde(rename = "compass", alias = "minecraft:compass")]
    Compass {
        target: java_assets_item_definition_CompassTarget,
        /// Whether to oscillate for some time around target before settling. Defaults to true.
        #[serde(skip_serializing_if = "Option::is_none")]
        wobble: Option<bool>,
    },
    #[serde(rename = "count", alias = "minecraft:count")]
    Count {
        /// If false, returns count clamped to `0..max_stack_size`.
        /// If true, returns count divided by the `max_stack_size` component, clamped to `0..1`.
        /// Defaults to true.
        #[serde(skip_serializing_if = "Option::is_none")]
        normalize: Option<bool>,
    },
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData {
        /// The index of the `floats` list in the `custom_model_data` component. Defaults to 0.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        index: Option<i32>,
    },
    #[serde(rename = "damage", alias = "minecraft:damage")]
    Damage {
        /// If false, returns value of damage, clamped to `0..max_damage`.
        /// If true, returns value of damage divided by the `max_damage` component, clamped to `0..1`.
        /// Defaults to true.
        #[serde(skip_serializing_if = "Option::is_none")]
        normalize: Option<bool>,
    },
    #[serde(rename = "time", alias = "minecraft:time")]
    Time {
        source: java_assets_item_definition_TimeSource,
        /// Whether to oscillate for some time around target before settling. Defaults to true.
        #[serde(skip_serializing_if = "Option::is_none")]
        wobble: Option<bool>,
    },
    #[serde(rename = "use_cycle", alias = "minecraft:use_cycle")]
    UseCycle {
        /// returns remaining item use ticks modulo `period`.
        /// Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        period: Option<f32>,
    },
    #[serde(rename = "use_duration", alias = "minecraft:use_duration")]
    UseDuration {
        /// If true, returns remaining item use ticks.
        /// If false, returns item use ticks so far.
        /// Defaults to false.
        #[serde(skip_serializing_if = "Option::is_none")]
        remaining: Option<bool>,
    },
    #[serde(rename = "bundle/fullness", alias = "minecraft:bundle/fullness")]
    BundleFullness {},
    #[serde(rename = "cooldown", alias = "minecraft:cooldown")]
    Cooldown {},
    #[serde(rename = "crossbow/pull", alias = "minecraft:crossbow/pull")]
    CrossbowPull {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_item_definition_SelectCase<T> {
    pub when: JavaAssetsItemDefinitionSelectCaseWhen<T>,
    pub model: java_assets_item_definition_ItemModel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "property")]
pub enum java_assets_item_definition_SelectPropertyType {
    #[serde(rename = "block_state", alias = "minecraft:block_state")]
    BlockState {
        block_state_property: JavaAssetsItemDefinitionSelectPropertyTypeBlockStateBlockStateProperty,
        cases: Vec<java_assets_item_definition_SelectCase<String>>,
    },
    #[serde(rename = "charge_type", alias = "minecraft:charge_type")]
    ChargeType {
        cases: Vec<java_assets_item_definition_SelectCase<java_assets_item_definition_CrossbowChargeType>>,
    },
    #[serde(rename = "component", alias = "minecraft:component")]
    Component {
        /// Identifier for the `data_component_type` registry.
        /// The component type to check the values of.
        /// If the selected value comes from a registry that the client doesn't have access to,
        /// the entry will be silently ignored.
        component: crate::resource_pack::identifier::Identifier,
        cases: serde_json::Value,
    },
    #[serde(rename = "context_dimension", alias = "minecraft:context_dimension")]
    ContextDimension {
        /// Identifier for the `dimension` registry.
        cases: Vec<java_assets_item_definition_SelectCase<crate::resource_pack::identifier::Identifier>>,
    },
    #[serde(rename = "context_entity_type", alias = "minecraft:context_entity_type")]
    ContextEntityType {
        /// Identifier for the `entity_type` registry.
        cases: Vec<java_assets_item_definition_SelectCase<crate::resource_pack::identifier::Identifier>>,
    },
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData {
        /// The index of the `strings` list in the `custom_model_data` component. Defaults to 0.
        #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
        index: Option<i32>,
        cases: Vec<java_assets_item_definition_SelectCase<String>>,
    },
    #[serde(rename = "display_context", alias = "minecraft:display_context")]
    DisplayContext {
        cases: Vec<java_assets_item_definition_SelectCase<java_assets_model_ItemDisplayContext>>,
    },
    #[serde(rename = "local_time", alias = "minecraft:local_time")]
    LocalTime {
        /// Format to use for time formatting.
        /// Examples: `yyyy-MM-dd`, `HH:mm:ss`.
        pattern: String,
        /// Defaults to the root locale.
        /// Examples: `en_US`, `cs_AU@numbers=thai;calendar=japanese`.
        #[serde(skip_serializing_if = "Option::is_none")]
        locale: Option<String>,
        /// Defaults to the timezone set on the client.
        /// Examples: `Europe/Stockholm`, `GMT+0:45`.
        #[serde(skip_serializing_if = "Option::is_none")]
        time_zone: Option<String>,
        cases: Vec<java_assets_item_definition_SelectCase<String>>,
    },
    #[serde(rename = "main_hand", alias = "minecraft:main_hand")]
    MainHand {
        cases: Vec<java_assets_item_definition_SelectCase<java_util_avatar_HumanoidArm>>,
    },
    #[serde(rename = "trim_material", alias = "minecraft:trim_material")]
    TrimMaterial {
        /// Identifier for the `trim_material` registry.
        cases: Vec<java_assets_item_definition_SelectCase<crate::resource_pack::identifier::Identifier>>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum java_assets_item_definition_SpecialModelType {
    #[serde(rename = "banner", alias = "minecraft:banner")]
    Banner {
        color: java_util_color_DyeColor,
        /// Defaults to `ground`.
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        attachment: Option<java_assets_item_definition_BannerAttachment>,
    },
    #[serde(rename = "bed", alias = "minecraft:bed")]
    Bed {
        /// Identifier for the `texture` registry.
        texture: crate::resource_pack::identifier::TextureId,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        part: Option<java_assets_item_definition_BedPart>,
    },
    #[serde(rename = "book", alias = "minecraft:book")]
    Book {
        /// Angle in degrees between book cover and book centerline. \
        /// `0.0` for closed, `90.0` for open flat.
        open_angle: f32,
        /// The position of the first page inside the book. \
        /// `0.0` for leftmost, `1.0` for rightmost.
        page1: f32,
        /// The position of the second page inside the book. \
        /// `0.0` for leftmost, `1.0` for rightmost.
        page2: f32,
    },
    #[serde(rename = "chest", alias = "minecraft:chest")]
    Chest {
        /// Identifier for the `texture` registry.
        texture: crate::resource_pack::identifier::TextureId,
        /// Defaults to `0`.
        #[serde(skip_serializing_if = "Option::is_none")]
        openness: Option<f32>,
        /// Defaults to `single`.
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        chest_type: Option<java_assets_item_definition_ChestType>,
    },
    #[serde(rename = "copper_golem_statue", alias = "minecraft:copper_golem_statue")]
    CopperGolemStatue {
        pose: java_assets_item_definition_CopperGolemStatuePose,
        texture: String,
    },
    #[serde(rename = "end_cube", alias = "minecraft:end_cube")]
    EndCube {
        effect: java_assets_item_definition_EndCubeEffectType,
    },
    #[serde(rename = "hanging_sign", alias = "minecraft:hanging_sign")]
    HangingSign {
        wood_type: java_assets_item_definition_WoodType,
        /// Identifier for the `texture` registry.
        #[serde(skip_serializing_if = "Option::is_none")]
        texture: Option<crate::resource_pack::identifier::TextureId>,
        /// Defaults to `ceiling_middle`.
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        attachment: Option<java_assets_item_definition_HangingSignAttachment>,
    },
    #[serde(rename = "head", alias = "minecraft:head")]
    Head {
        kind: java_assets_item_definition_HeadType,
        /// Identifier for the `texture` registry.
        /// Texture to use instead of the texture from `kind`.
        /// If present, will ignore the `profile` component.
        #[serde(skip_serializing_if = "Option::is_none")]
        texture: Option<crate::resource_pack::identifier::TextureId>,
        /// Controls the animation time for piglin and dragon heads. Defaults to `0`.
        #[serde(skip_serializing_if = "Option::is_none")]
        animation: Option<f32>,
    },
    #[serde(rename = "shulker_box", alias = "minecraft:shulker_box")]
    ShulkerBox {
        /// Identifier for the `texture` registry.
        texture: crate::resource_pack::identifier::TextureId,
        #[serde(skip_serializing_if = "Option::is_none")]
        openness: Option<f32>,
        /// Defaults to `up`.
        #[serde(skip_serializing_if = "is_none_or_newer_than_1_21_11")]
        orientation: Option<java_util_direction_Direction>,
    },
    #[serde(rename = "standing_sign", alias = "minecraft:standing_sign")]
    StandingSign {
        wood_type: java_assets_item_definition_WoodType,
        /// Identifier for the `texture` registry.
        #[serde(skip_serializing_if = "Option::is_none")]
        texture: Option<crate::resource_pack::identifier::TextureId>,
        /// There is an extra "e" in the field name. See MC-307498. \
        /// Defaults to `ground`.
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1")]
        attachement: Option<java_assets_item_definition_StandingSignAttachment>,
    },
    #[serde(rename = "bell", alias = "minecraft:bell")]
    Bell {},
    #[serde(rename = "conduit", alias = "minecraft:conduit")]
    Conduit {},
    #[serde(rename = "decorated_pot", alias = "minecraft:decorated_pot")]
    DecoratedPot {},
    #[serde(rename = "player_head", alias = "minecraft:player_head")]
    PlayerHead {},
    #[serde(rename = "shield", alias = "minecraft:shield")]
    Shield {},
    #[serde(rename = "trident", alias = "minecraft:trident")]
    Trident {},
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_StandingSignAttachment {
    #[serde(rename = "wall", alias = "minecraft:wall")]
    Wall,
    #[serde(rename = "ground", alias = "minecraft:ground")]
    Ground,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_TimeSource {
    #[serde(rename = "daytime", alias = "minecraft:daytime")]
    Daytime,
    #[serde(rename = "moon_phase", alias = "minecraft:moon_phase")]
    MoonPhase,
    #[serde(rename = "random", alias = "minecraft:random")]
    Random,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_item_definition_WoodType {
    #[serde(rename = "oak", alias = "minecraft:oak")]
    Oak,
    #[serde(rename = "spruce", alias = "minecraft:spruce")]
    Spruce,
    #[serde(rename = "birch", alias = "minecraft:birch")]
    Birch,
    #[serde(rename = "acacia", alias = "minecraft:acacia")]
    Acacia,
    #[serde(rename = "cherry", alias = "minecraft:cherry")]
    Cherry,
    #[serde(rename = "jungle", alias = "minecraft:jungle")]
    Jungle,
    #[serde(rename = "dark_oak", alias = "minecraft:dark_oak")]
    DarkOak,
    #[serde(rename = "pale_oak", alias = "minecraft:pale_oak")]
    PaleOak,
    #[serde(rename = "mangrove", alias = "minecraft:mangrove")]
    Mangrove,
    #[serde(rename = "bamboo", alias = "minecraft:bamboo")]
    Bamboo,
    #[serde(rename = "crimson", alias = "minecraft:crimson")]
    Crimson,
    #[serde(rename = "warped", alias = "minecraft:warped")]
    Warped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_assets_model_ItemDisplayContext {
    #[serde(rename = "none", alias = "minecraft:none")]
    None,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_util_avatar_HumanoidArm {
    #[serde(rename = "left", alias = "minecraft:left")]
    Left,
    #[serde(rename = "right", alias = "minecraft:right")]
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_util_color_DyeColor {
    #[serde(rename = "white", alias = "minecraft:white")]
    White,
    #[serde(rename = "orange", alias = "minecraft:orange")]
    Orange,
    #[serde(rename = "magenta", alias = "minecraft:magenta")]
    Magenta,
    #[serde(rename = "light_blue", alias = "minecraft:light_blue")]
    LightBlue,
    #[serde(rename = "yellow", alias = "minecraft:yellow")]
    Yellow,
    #[serde(rename = "lime", alias = "minecraft:lime")]
    Lime,
    #[serde(rename = "pink", alias = "minecraft:pink")]
    Pink,
    #[serde(rename = "gray", alias = "minecraft:gray")]
    Gray,
    #[serde(rename = "light_gray", alias = "minecraft:light_gray")]
    LightGray,
    #[serde(rename = "cyan", alias = "minecraft:cyan")]
    Cyan,
    #[serde(rename = "purple", alias = "minecraft:purple")]
    Purple,
    #[serde(rename = "blue", alias = "minecraft:blue")]
    Blue,
    #[serde(rename = "brown", alias = "minecraft:brown")]
    Brown,
    #[serde(rename = "green", alias = "minecraft:green")]
    Green,
    #[serde(rename = "red", alias = "minecraft:red")]
    Red,
    #[serde(rename = "black", alias = "minecraft:black")]
    Black,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_util_color_RGB {
    Integer1(i32),
    List22([f32; 3]),
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum java_util_text_Keybind {
    #[serde(rename = "key.advancements", alias = "minecraft:key.advancements")]
    Advancements,
    #[serde(rename = "key.attack", alias = "minecraft:key.attack")]
    Attack,
    #[serde(rename = "key.back", alias = "minecraft:key.back")]
    Back,
    #[serde(rename = "key.chat", alias = "minecraft:key.chat")]
    Chat,
    #[serde(rename = "key.command", alias = "minecraft:key.command")]
    Command,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.clearChat", alias = "minecraft:key.debug.clearChat")]
    DebugClearChat,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.copyLocation", alias = "minecraft:key.debug.copyLocation")]
    DebugCopyLocation,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.copyRecreateCommand", alias = "minecraft:key.debug.copyRecreateCommand")]
    DebugCopyRecreateCommand,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.crash", alias = "minecraft:key.debug.crash")]
    DebugCrash,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.debugOptions", alias = "minecraft:key.debug.debugOptions")]
    DebugOptionsMenu,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.dumpDynamicTextures", alias = "minecraft:key.debug.dumpDynamicTextures")]
    DebugDumpDynamicTextures,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.dumpVersion", alias = "minecraft:key.debug.dumpVersion")]
    DebugDumpVersion,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.focusPause", alias = "minecraft:key.debug.focusPause")]
    DebugFocusPause,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.fpsCharts", alias = "minecraft:key.debug.fpsCharts")]
    DebugFpsCharts,
    /// Available since Minecraft 26.3.
    #[serde(rename = "key.debug.improvedTransparency", alias = "minecraft:key.debug.improvedTransparency")]
    DebugImprovedTransparency,
    /// Available since Minecraft 26.1.
    #[serde(rename = "key.debug.lightmapTexture", alias = "minecraft:key.debug.lightmapTexture")]
    DebugLightmapTexture,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.modifier", alias = "minecraft:key.debug.modifier")]
    DebugModifier,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.networkCharts", alias = "minecraft:key.debug.networkCharts")]
    DebugNetworkCharts,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.overlay", alias = "minecraft:key.debug.overlay")]
    DebugOverlay,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.profiling", alias = "minecraft:key.debug.profiling")]
    DebugProfiling,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.profilingChart", alias = "minecraft:key.debug.profilingChart")]
    DebugProfilingChart,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.reloadChunk", alias = "minecraft:key.debug.reloadChunk")]
    DebugReloadChunk,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.reloadResourcePacks", alias = "minecraft:key.debug.reloadResourcePacks")]
    DebugReloadResourcePacks,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.showAdvancedTooltips", alias = "minecraft:key.debug.showAdvancedTooltips")]
    DebugShowAdvancedTooltips,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.showChunkBorders", alias = "minecraft:key.debug.showChunkBorders")]
    DebugShowChunkBorders,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.showHitboxes", alias = "minecraft:key.debug.showHitboxes")]
    DebugShowHitboxes,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.spectate", alias = "minecraft:key.debug.spectate")]
    DebugSpectate,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.debug.switchGameMode", alias = "minecraft:key.debug.switchGameMode")]
    DebugSwitchGameMode,
    #[serde(rename = "key.drop", alias = "minecraft:key.drop")]
    Drop,
    #[serde(rename = "key.forward", alias = "minecraft:key.forward")]
    Forward,
    /// Available since Minecraft 26.2.
    #[serde(rename = "key.friends", alias = "minecraft:key.friends")]
    Friends,
    #[serde(rename = "key.fullscreen", alias = "minecraft:key.fullscreen")]
    Fullscreen,
    #[serde(rename = "key.hotbar.1", alias = "minecraft:key.hotbar.1")]
    Hotbar1,
    #[serde(rename = "key.hotbar.2", alias = "minecraft:key.hotbar.2")]
    Hotbar2,
    #[serde(rename = "key.hotbar.3", alias = "minecraft:key.hotbar.3")]
    Hotbar3,
    #[serde(rename = "key.hotbar.4", alias = "minecraft:key.hotbar.4")]
    Hotbar4,
    #[serde(rename = "key.hotbar.5", alias = "minecraft:key.hotbar.5")]
    Hotbar5,
    #[serde(rename = "key.hotbar.6", alias = "minecraft:key.hotbar.6")]
    Hotbar6,
    #[serde(rename = "key.hotbar.7", alias = "minecraft:key.hotbar.7")]
    Hotbar7,
    #[serde(rename = "key.hotbar.8", alias = "minecraft:key.hotbar.8")]
    Hotbar8,
    #[serde(rename = "key.hotbar.9", alias = "minecraft:key.hotbar.9")]
    Hotbar9,
    #[serde(rename = "key.inventory", alias = "minecraft:key.inventory")]
    Inventory,
    #[serde(rename = "key.jump", alias = "minecraft:key.jump")]
    Jump,
    #[serde(rename = "key.left", alias = "minecraft:key.left")]
    Left,
    #[serde(rename = "key.loadToolbarActivator", alias = "minecraft:key.loadToolbarActivator")]
    LoadToolbarActivator,
    #[serde(rename = "key.pickItem", alias = "minecraft:key.pickItem")]
    PickItem,
    #[serde(rename = "key.playerlist", alias = "minecraft:key.playerlist")]
    Playerlist,
    /// Available since Minecraft 1.21.6.
    #[serde(rename = "key.quickActions", alias = "minecraft:key.quickActions")]
    QuickActions,
    #[serde(rename = "key.right", alias = "minecraft:key.right")]
    Right,
    #[serde(rename = "key.saveToolbarActivator", alias = "minecraft:key.saveToolbarActivator")]
    SaveToolbarActivator,
    #[serde(rename = "key.screenshot", alias = "minecraft:key.screenshot")]
    Screenshot,
    #[serde(rename = "key.smoothCamera", alias = "minecraft:key.smoothCamera")]
    SmoothCamera,
    #[serde(rename = "key.sneak", alias = "minecraft:key.sneak")]
    Sneak,
    /// Available since Minecraft 1.21.9.
    #[serde(rename = "key.spectatorHotbar", alias = "minecraft:key.spectatorHotbar")]
    SpectatorHotbar,
    #[serde(rename = "key.spectatorOutlines", alias = "minecraft:key.spectatorOutlines")]
    SpectatorOutlines,
    #[serde(rename = "key.sprint", alias = "minecraft:key.sprint")]
    Sprint,
    #[serde(rename = "key.swapOffhand", alias = "minecraft:key.swapOffhand")]
    SwapOffhand,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.toggleGui", alias = "minecraft:key.toggleGui")]
    ToggleGui,
    #[serde(rename = "key.togglePerspective", alias = "minecraft:key.togglePerspective")]
    TogglePerspective,
    /// Available since Minecraft 1.21.11.
    #[serde(rename = "key.toggleSpectatorShaderEffects", alias = "minecraft:key.toggleSpectatorShaderEffects")]
    ToggleSpectatorShaderEffects,
    #[serde(rename = "key.use", alias = "minecraft:key.use")]
    Use,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_world_entity_display_AxisAngle {
    /// Local position of the axis in [x, y, z].
    pub axis: [f32; 3],
    /// Angle to rotate around the axis in radians.
    pub angle: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_world_entity_display_Rotation {
    List11([f32; 4]),
    java_world_entity_display_AxisAngle2(java_world_entity_display_AxisAngle),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_world_entity_display_Transformation {
    Object11(JavaWorldEntityDisplayTransformationObject11Anonymous),
    Variant22(JavaWorldEntityDisplayTransformationVariant22),
}
