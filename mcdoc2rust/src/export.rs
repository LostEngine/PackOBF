/* AI Slop Notice
Code in this module (mcdoc2rust), which is not part of the PackOBF cargo workspace, was highly generated using LLMs.
All the code hasn't been reviewed by a real developer, and it may be terrible. However, the code that it generates is
manually verified and tested and is included manually in the PackOBF codebase. This code is provided for transparency,
but it does not reflect the rest of the codebase.
See more in mcdoc2rust/README.md
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root container for the compiled mcdoc JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McdocFile {
    /// Git commit reference / version hash.
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,

    /// Map of canonical type paths to their type definition.
    /// e.g. `::java::assets::atlas::Atlas`
    pub mcdoc: HashMap<String, TypeDef>,

    /// Map of dispatcher registries to their variant-to-type mappings.
    /// e.g. `minecraft:block` -> { "acacia_sign": TypeDef }
    #[serde(rename = "mcdoc/dispatcher")]
    pub mcdoc_dispatcher: HashMap<String, HashMap<String, TypeDef>>,
}

/// Represents any mcdoc type definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeDef {
    Struct {
        fields: Vec<StructField>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Union {
        members: Vec<TypeDef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    List {
        item: Box<TypeDef>,
        #[serde(
            default,
            rename = "lengthRange",
            skip_serializing_if = "Option::is_none"
        )]
        length_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Tuple {
        items: Vec<TypeDef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Enum {
        #[serde(rename = "enumKind")]
        enum_kind: String,
        values: Vec<EnumMember>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Reference {
        path: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Dispatcher {
        #[serde(rename = "parallelIndices")]
        parallel_indices: Vec<IndexAccessor>,
        registry: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Template {
        child: Box<TypeDef>,
        #[serde(rename = "typeParams")]
        type_params: Vec<TypeParam>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Concrete {
        child: Box<TypeDef>,
        #[serde(rename = "typeArgs")]
        type_args: Vec<TypeDef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Indexed {
        child: Box<TypeDef>,
        #[serde(rename = "parallelIndices")]
        parallel_indices: Vec<IndexAccessor>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Literal {
        value: LiteralValue,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    // Primitives
    String {
        #[serde(
            default,
            rename = "lengthRange",
            skip_serializing_if = "Option::is_none"
        )]
        length_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Int {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Float {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Double {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Byte {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Short {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Long {
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Boolean {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    IntArray {
        #[serde(
            default,
            rename = "lengthRange",
            skip_serializing_if = "Option::is_none"
        )]
        length_range: Option<Range>,
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    ByteArray {
        #[serde(
            default,
            rename = "lengthRange",
            skip_serializing_if = "Option::is_none"
        )]
        length_range: Option<Range>,
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    LongArray {
        #[serde(
            default,
            rename = "lengthRange",
            skip_serializing_if = "Option::is_none"
        )]
        length_range: Option<Range>,
        #[serde(
            default,
            rename = "valueRange",
            skip_serializing_if = "Option::is_none"
        )]
        value_range: Option<Range>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Any {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
}

/// A field within a `struct`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StructField {
    Pair {
        key: FieldKey,
        #[serde(rename = "type")]
        field_type: TypeDef,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        optional: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        desc: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    Spread {
        #[serde(rename = "type")]
        field_type: TypeDef,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        desc: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
}

/// The key of a struct field pair. Can be a static string name (e.g. `"id"`)
/// or a dynamic type definition matching a key schema (e.g. `{"kind": "string", ...}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldKey {
    String(String),
    Type(Box<TypeDef>),
}

/// Numerical bounds for length or values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub kind: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Generic type parameter path for templates (e.g. `::java::assets::equipment::T`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParam {
    pub path: String,
}

/// Literal values used for constant types or attribute metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum LiteralValue {
    String(String),
    Boolean(bool),
    Int(i64),
    Float(f64),
    Double(f64),
    Byte(i8),
    Short(i16),
    Long(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Attribute {
    // --- Reference only ---
    Bitfield {
        value: ReferenceAttr,
    },

    // --- Literal only ---
    Color {
        value: LiteralAttr,
    },
    DispatcherKey {
        value: LiteralAttr,
    },
    DivisibleBy {
        value: LiteralAttr,
    },
    MatchRegex {
        value: LiteralAttr,
    },
    Since {
        value: LiteralAttr,
    },
    Until {
        value: LiteralAttr,
    },

    // --- Tree only ---
    Command {
        value: TreeAttr,
    },
    GameRule {
        value: TreeAttr,
    },
    Integer {
        value: TreeAttr,
    },
    Permutation {
        value: TreeAttr,
    },
    TextureSlot {
        value: TreeAttr,
    },
    Vector {
        value: TreeAttr,
    },

    // --- Literal | None ---
    Deprecated {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<LiteralAttr>,
    },

    // --- Tree | None ---
    CraftingIngredient {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<TreeAttr>,
    },
    Criterion {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<TreeAttr>,
    },
    Entity {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<TreeAttr>,
    },
    TranslationKey {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<TreeAttr>,
    },

    // --- Dispatcher | None ---
    NbtPath {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<DispatcherAttr>,
    },

    // --- Literal | Tree | None ---
    Id {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<LiteralOrTreeAttr>,
    },

    // --- Dispatcher | Reference ---
    Nbt {
        value: DispatcherOrReferenceAttr,
    },

    // --- None (Unit Variants) ---
    BlockPredicate,
    Canonical,
    ItemSlots,
    Objective,
    PackFormat,
    Random,
    RegexPattern,
    ScoreHolder,
    Tag,
    Team,
    TextComponent,
    TimePattern,
    TranslationValue,
    Url,
    Uuid,

    /// Catch-all fallback for unrecognized attributes.
    #[serde(other)]
    Unknown,
}

/// Attribute value that must be a `{"kind": "reference", "path": "..."}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceAttr {
    Reference { path: String },
}

/// Attribute value that must be a `{"kind": "literal", "value": ...}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LiteralAttr {
    Literal { value: LiteralValue },
}

impl LiteralAttr {
    pub fn value(&self) -> &LiteralValue {
        match self {
            Self::Literal { value } => value,
        }
    }
}

/// Attribute value that must be a `{"kind": "tree", "values": {...}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TreeAttr {
    Tree { values: HashMap<String, AttrValue> },
}

/// Attribute value that must be a `{"kind": "dispatcher", ...}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DispatcherAttr {
    Dispatcher {
        #[serde(rename = "parallelIndices")]
        parallel_indices: Vec<IndexAccessor>,
        registry: String,
    },
}

/// Attribute value that can be either a `Literal` or a `Tree` (e.g. for `@id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LiteralOrTreeAttr {
    Literal { value: LiteralValue },
    Tree { values: HashMap<String, AttrValue> },
}

/// Attribute value that can be either a `Dispatcher` or a `Reference` (e.g. for `@nbt`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DispatcherOrReferenceAttr {
    Dispatcher {
        #[serde(rename = "parallelIndices")]
        parallel_indices: Vec<IndexAccessor>,
        registry: String,
    },
    Reference {
        path: String,
    },
}

/// Recursive value container used inside `TreeAttr` values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttrValue {
    Literal {
        value: LiteralValue,
    },
    Tree {
        values: HashMap<String, AttrValue>,
    },
    Dispatcher {
        #[serde(rename = "parallelIndices")]
        parallel_indices: Vec<IndexAccessor>,
        registry: String,
    },
    Reference {
        path: String,
    },
}

/// Accessor indices for dispatchers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IndexAccessor {
    Dynamic { accessor: Vec<AccessorToken> },
    Static { value: String },
}

/// Elements inside a dynamic accessor path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AccessorToken {
    String(String),
    Keyword { keyword: String },
}

/// Enum entry definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumMember {
    pub identifier: String,
    pub value: EnumValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
}

/// Raw representation of enum values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EnumValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}
