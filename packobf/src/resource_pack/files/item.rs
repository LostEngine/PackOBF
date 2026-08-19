use crate::resource_pack::identifier::{Identifier, ModelId};
use crate::utils::clean_json_numbers;
use crate::version;
use serde::{Deserialize, Serialize};

/// https://github.com/SpyglassMC/vanilla-mcdoc/blob/main/java/assets/item_definition.mcdoc

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(skip)]
    pub overlay: String,
    #[serde(skip)]
    pub identifier: Identifier,

    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub hand_animation_on_swap: bool,
    #[serde(default, skip_serializing_if = "is_false_or_older_than_1_21_6")]
    pub oversized_in_gui: bool,
    #[serde(
        default = "default_one",
        skip_serializing_if = "is_one_or_older_than_1_21_11"
    )]
    pub swap_animation_scale: f32,
    pub model: Model,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Model {
    #[serde(rename = "model", alias = "minecraft:model")]
    Model {
        model: ModelId,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tints: Vec<TintSource>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
    },
    #[serde(rename = "composite", alias = "minecraft:composite")]
    Composite {
        models: Vec<Model>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
    },
    #[serde(rename = "condition", alias = "minecraft:condition")]
    Condition {
        property: String,
        on_true: Box<Model>,
        on_false: Box<Model>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
        #[serde(flatten)]
        extra: serde_json::Value,
    },
    #[serde(rename = "select", alias = "minecraft:select")]
    Select {
        property: String,
        cases: Vec<SelectCase>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fallback: Option<Box<Model>>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
        #[serde(flatten)]
        extra: serde_json::Value,
    },
    #[serde(rename = "range_dispatch", alias = "minecraft:range_dispatch")]
    RangeDispatch {
        property: String,
        #[serde(default = "default_one", skip_serializing_if = "is_one")]
        scale: f32,
        entries: Vec<RangeEntry>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fallback: Option<Box<Model>>,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
        #[serde(flatten)]
        extra: serde_json::Value,
    },
    #[serde(rename = "special", alias = "minecraft:special")]
    Special {
        base: ModelId,
        model: SpecialModelData,
        #[serde(skip_serializing_if = "is_none_or_older_than_26_1", flatten)]
        transformation: Option<Transformation>,
    },
    #[serde(rename = "empty", alias = "minecraft:empty")]
    Empty {},
    #[serde(
        rename = "bundle/selected_item",
        alias = "minecraft:bundle/selected_item"
    )]
    BundleSelectedItem {},
}

// TODO: Convert Object to list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Transformation {
    List { transformation: [f32; 16] },
    Object { transformation: FullTransformation },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullTransformation {
    left_rotation: Quaternion,
    right_rotation: Quaternion,
    scale: [f32; 3],
    translation: [f32; 3],
}

// TODO: Convert Object to list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Quaternion {
    List { quaternion: [f32; 4] },
    Object { quaternion: FullQuaternion },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullQuaternion {
    axis: [f32; 3],
    angle: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelectCase {
    pub when: serde_json::Value, // Can be String or Array of Strings
    pub model: Model,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RangeEntry {
    pub threshold: f32,
    pub model: Model,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TintSource {
    #[serde(rename = "constant", alias = "minecraft:constant")]
    Constant { value: serde_json::Value },
    #[serde(rename = "dye", alias = "minecraft:dye")]
    Dye { default: serde_json::Value },
    #[serde(rename = "firework", alias = "minecraft:firework")]
    Firework { default: serde_json::Value },
    #[serde(rename = "grass", alias = "minecraft:grass")]
    Grass { temperature: f32, downfall: f32 },
    #[serde(rename = "map_color", alias = "minecraft:map_color")]
    MapColor { default: serde_json::Value },
    #[serde(rename = "potion", alias = "minecraft:potion")]
    Potion { default: serde_json::Value },
    #[serde(rename = "team", alias = "minecraft:team")]
    Team { default: serde_json::Value },
    #[serde(rename = "custom_model_data", alias = "minecraft:custom_model_data")]
    CustomModelData {
        #[serde(default)]
        index: i32,
        default: serde_json::Value,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SpecialModelData {
    #[serde(rename = "banner", alias = "minecraft:banner")]
    Banner {
        #[serde(
            default = "default_ground",
            skip_serializing_if = "version::is_older_than_26_1"
        )]
        attachment: String,
        color: String,
    },
    #[serde(rename = "bed", alias = "minecraft:bed")]
    Bed {
        #[serde(skip_serializing_if = "version::is_not_between_26_1_and_26_2")]
        part: String,
        #[serde(skip_serializing_if = "version::is_newer_than_26_1")]
        texture: Identifier
    },
    // bed atlas
    #[serde(rename = "bell", alias = "minecraft:bell")]
    Bell {},
    #[serde(rename = "book", alias = "minecraft:book")]
    Book {
        #[serde(default, skip_serializing_if = "version::is_older_than_26_1")]
        open_angle: f32,
        #[serde(default, skip_serializing_if = "version::is_older_than_26_1")]
        page1: f32,
        #[serde(default, skip_serializing_if = "version::is_older_than_26_1")]
        page2: f32,
    },
    #[serde(rename = "conduit", alias = "minecraft:conduit")]
    Conduit {},
    #[serde(rename = "chest", alias = "minecraft:chest")]
    Chest {
        texture: Identifier, // chest atlas
        #[serde(default = "default_single", skip_serializing_if = "version::is_older_than_26_1")]
        chest_type: String,
        #[serde(default)]
        openness: f32,
    },
    #[serde(
        rename = "copper_golem_statue",
        alias = "minecraft:copper_golem_statue"
    )]
    CopperGolemStatue {
        texture: String, // with the ".png" suffix
        pose: String,
    },
    #[serde(rename = "decorated_pot", alias = "minecraft:decorated_pot")]
    DecoratedPot {},
    #[serde(rename = "end_cube", alias = "minecraft:end_cube")]
    EndCube {
        #[serde(skip_serializing_if = "version::is_older_than_26_1")]
        effect: String
    },
    #[serde(rename = "head", alias = "minecraft:head")]
    Head {
        kind: String, // without textures/entity/ prefix and .png suffix
        #[serde(skip_serializing_if = "Option::is_none")]
        texture: Option<String>,
        #[serde(default)]
        animation: f32,
    },
    #[serde(rename = "player_head", alias = "minecraft:player_head")]
    PlayerHead {},
    #[serde(rename = "shield", alias = "minecraft:shield")]
    Shield {},
    #[serde(rename = "shulker_box", alias = "minecraft:shulker_box")]
    ShulkerBox {
        texture: String,
        #[serde(default)]
        openness: f32,
        #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
        orientation: Option<String>,
    },
    #[serde(rename = "standing_sign", alias = "minecraft:standing_sign")]
    StandingSign {
        #[serde(default = "default_ground", skip_serializing_if = "version::is_not_between_26_1_and_26_2")]
        attachment: String,
        #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
        wood_type: Option<String>,
        #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
        texture: Option<Identifier>, // signs atlas
    },
    #[serde(rename = "hanging_sign", alias = "minecraft:hanging_sign")]
    HangingSign {
        #[serde(default = "default_ceiling_middle", skip_serializing_if = "version::is_not_between_26_1_and_26_2")]
        attachment: String,
        #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
        wood_type: Option<String>,
        #[serde(skip_serializing_if = "is_none_or_newer_than_26_1")]
        texture: Option<Identifier>, // signs atlas
    },
    #[serde(rename = "trident", alias = "minecraft:trident")]
    Trident {},
}

// Helpers for default values
fn default_true() -> bool {
    true
}
fn is_true(b: &bool) -> bool {
    *b
}
fn default_one() -> f32 {
    1.0
}
fn is_one(f: &f32) -> bool {
    (*f - 1.0).abs() < f32::EPSILON
}
fn default_ground() -> String {
    "ground".to_string()
}

fn default_ceiling_middle() -> String {
    "ceiling_middle".to_string()
}
fn default_single() -> String {
    "single".to_string()
}

pub fn is_false_or_older_than_1_21_6(value: &bool) -> bool {
    !*value || version::is_older_than_1_21_4(&())
}

pub fn is_one_or_older_than_1_21_11(f: &f32) -> bool {
    (*f - 1.0).abs() < f32::EPSILON || version::is_older_than_1_21_11(&())
}

pub fn is_none_or_older_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_older_than_26_1(&())
}

pub fn is_none_or_newer_than_26_1<T>(value: &Option<T>) -> bool {
    value.is_none() || version::is_newer_than_26_1(&())
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = serde_json::to_value(self).map_err(|_| std::fmt::Error)?;
        clean_json_numbers(&mut val);
        write!(
            f,
            "{}",
            serde_json::to_string(&val).map_err(|_| std::fmt::Error)?
        )
    }
}

impl Item {
    pub fn from_json(
        overlay: impl Into<String>,
        identifier: Identifier,
        json: &str,
    ) -> Result<Self, serde_json::Error> {
        let mut item: Item = serde_json::from_str(json)?;

        item.overlay = overlay.into();
        item.identifier = identifier;

        Ok(item)
    }

    pub fn path(&self) -> String {
        let prefix = if self.overlay.is_empty() {
            "".to_string()
        } else {
            format!("{}/", self.overlay)
        };
        format!(
            "{}assets/{}/items/{}.json",
            prefix, self.identifier.namespace, self.identifier.path
        )
    }
}
