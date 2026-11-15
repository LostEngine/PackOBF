use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::*;

#[inline]
pub fn is_none_or_older_than_1_21_11<T>(val: &Option<T>) -> bool {
    val.is_none() || crate::version::is_older_than_1_21_11(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionBlockStateDefinitionObject11Anonymous {
    pub variants: HashMap<String, java_assets_block_state_definition_ModelVariant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionBlockStateDefinitionObject22Anonymous {
    pub multipart: Vec<JavaAssetsBlockStateDefinitionBlockStateDefinitionObject22AnonymousMultipartAnonymous>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionBlockStateDefinitionObject22AnonymousMultipartAnonymous {
    /// One condition or an array where at least one condition must apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<java_assets_block_state_definition_MultiPartCondition>,
    pub apply: java_assets_block_state_definition_ModelVariant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionModelVariantList22Anonymous {
    /// Identifier for the `model` registry.
    pub model: crate::resource_pack::identifier::ModelId,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub z: Option<i32>,
    /// If set to `true`, the textures are not rotated with the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uvlock: Option<bool>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionMultiPartConditionObject11Anonymous {
    pub OR: Vec<java_assets_block_state_definition_MultiPartCondition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaAssetsBlockStateDefinitionMultiPartConditionObject22Anonymous {
    pub AND: Vec<java_assets_block_state_definition_MultiPartCondition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_block_state_definition_BlockStateDefinition {
    Object11(JavaAssetsBlockStateDefinitionBlockStateDefinitionObject11Anonymous),
    Object22(JavaAssetsBlockStateDefinitionBlockStateDefinitionObject22Anonymous),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_block_state_definition_ModelVariant {
    java_assets_block_state_definition_ModelVariantBase1(java_assets_block_state_definition_ModelVariantBase),
    List22(Vec<JavaAssetsBlockStateDefinitionModelVariantList22Anonymous>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct java_assets_block_state_definition_ModelVariantBase {
    /// Identifier for the `model` registry.
    pub model: crate::resource_pack::identifier::ModelId,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_i32", default, skip_serializing_if = "is_none_or_older_than_1_21_11")]
    pub z: Option<i32>,
    /// If set to `true`, the textures are not rotated with the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uvlock: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum java_assets_block_state_definition_MultiPartCondition {
    Object11(JavaAssetsBlockStateDefinitionMultiPartConditionObject11Anonymous),
    Object22(JavaAssetsBlockStateDefinitionMultiPartConditionObject22Anonymous),
    Object33(HashMap<String, String>),
}
