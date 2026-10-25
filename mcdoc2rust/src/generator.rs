/* AI Slop Notice
Code in this module (mcdoc2rust), which is not part of the PackOBF cargo workspace, was highly generated using LLMs.
All the code hasn't been reviewed by a real developer, and it may be terrible. However, the code that it generates is
manually verified and tested and is included manually in the PackOBF codebase. This code is provided for transparency,
but it does not reflect the rest of the codebase.
See more in mcdoc2rust/README.md
*/

use crate::export::*;
use std::collections::{BTreeMap, BTreeSet};

// =========================================================================
// Version Management
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn parse(s: &str) -> Self {
        let s = match s {
            "1.21.3" => "1.21.4",
            "1.21.8" => "1.21.9",
            "1.21.10" => "1.21.11",
            other => other,
        };

        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        match parts.as_slice() {
            [maj] => Self {
                major: *maj,
                minor: 0,
                patch: 0,
            },
            [maj, min] => Self {
                major: *maj,
                minor: *min,
                patch: 0,
            },
            [maj, min, pat, ..] => Self {
                major: *maj,
                minor: *min,
                patch: *pat,
            },
            [] => Self {
                major: 0,
                minor: 0,
                patch: 0,
            },
        }
    }
}

pub const SUPPORTED_MILESTONES: &[(&str, &str)] = &[
    ("1.21.1", "1_21_1"),
    ("1.21.2", "1_21_2"),
    ("1.21.4", "1_21_4"),
    ("1.21.5", "1_21_5"),
    ("1.21.6", "1_21_6"),
    ("1.21.7", "1_21_7"),
    ("1.21.9", "1_21_9"),
    ("1.21.11", "1_21_11"),
    ("26.1", "26_1"),
    ("26.2", "26_2"),
    ("26.3", "26_3"),
];

pub const MIN_VERSION: SemVer = SemVer {
    major: 1,
    minor: 21,
    patch: 1,
};

// =========================================================================
// Code Generator
// =========================================================================

pub struct Generator<'a> {
    mcdoc: &'a McdocFile,
    value_overrides: BTreeSet<String>,
    active_type_params: Vec<String>,
    use_specialized_identifier_types: bool,
    generated_types: BTreeMap<String, String>,
    in_progress: BTreeSet<String>,
    required_helpers: BTreeSet<String>,
}

impl<'a> Generator<'a> {
    pub fn new(mcdoc: &'a McdocFile) -> Self {
        Self {
            mcdoc,
            value_overrides: BTreeSet::new(),
            active_type_params: Vec::new(),
            use_specialized_identifier_types: true,
            generated_types: BTreeMap::new(),
            in_progress: BTreeSet::new(),
            required_helpers: BTreeSet::new(),
        }
    }

    pub fn with_value_overrides(mut self, type_names: &[&str]) -> Self {
        self.value_overrides
            .extend(type_names.iter().map(|name| (*name).to_string()));
        self
    }

    pub fn with_specialized_identifier_types(mut self, enabled: bool) -> Self {
        self.use_specialized_identifier_types = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn generate_all(mut self) -> String {
        let mut paths: Vec<_> = self.mcdoc.mcdoc.keys().cloned().collect();
        paths.sort();

        for path in paths {
            let type_def = self.mcdoc.mcdoc.get(&path).unwrap().clone();
            let rust_name = type_name_from_path(&path);
            self.generate_top_level_type(&rust_name, &type_def);
        }

        self.assemble_output()
    }

    #[allow(dead_code)]
    pub fn generate_for_path(self, path: &str) -> String {
        self.generate_for_path_with_type_closures(path).0
    }

    pub fn generate_for_path_with_type_closures(
        mut self,
        path: &str,
    ) -> (String, Vec<(String, usize)>) {
        if let Some(type_def) = self.mcdoc.mcdoc.get(path).cloned() {
            let rust_name = type_name_from_path(path);
            self.generate_top_level_type(&rust_name, &type_def);
        }
        let closures = self.generated_type_closures();
        (self.assemble_output(), closures)
    }

    fn generated_type_closures(&self) -> Vec<(String, usize)> {
        let type_names: BTreeSet<_> = self
            .generated_types
            .iter()
            .filter(|(_, code)| !code.is_empty())
            .map(|(name, _)| name.clone())
            .collect();

        type_names
            .iter()
            .map(|name| {
                let mut reachable = BTreeSet::new();
                collect_generated_type_dependencies(
                    name,
                    &self.generated_types,
                    &type_names,
                    &mut reachable,
                );
                (name.clone(), reachable.len())
            })
            .collect()
    }

    fn assemble_output(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "#![allow(dead_code, unused_imports, non_snake_case, non_camel_case_types)]\n",
        );
        out.push_str("// Auto-generated by mcdoc code generator.\n");
        out.push_str("use serde::{Deserialize, Serialize};\n");
        out.push_str("use std::collections::HashMap;\n\n");

        out.push_str("// --- Helper Deserializers for Integers (fallback via f64) ---\n\n");
        out.push_str(
            r#"macro_rules! impl_int_deserializers {
    ($ty:ident, $single_fn:ident, $opt_fn:ident, $vec_fn:ident, $opt_vec_fn:ident) => {
        pub fn $single_fn<'de, D>(deserializer: D) -> Result<$ty, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Helper {
                Int($ty),
                Float(f64),
                Str(String),
            }
            match Helper::deserialize(deserializer)? {
                Helper::Int(i) => Ok(i),
                Helper::Float(f) => Ok(f as $ty),
                Helper::Str(s) => s
                    .parse::<$ty>()
                    .or_else(|_| s.parse::<f64>().map(|f| f as $ty))
                    .map_err(serde::de::Error::custom),
            }
        }

        pub fn $opt_fn<'de, D>(deserializer: D) -> Result<Option<$ty>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Helper {
                Int($ty),
                Float(f64),
                Str(String),
            }
            match Option::<Helper>::deserialize(deserializer)? {
                Some(Helper::Int(i)) => Ok(Some(i)),
                Some(Helper::Float(f)) => Ok(Some(f as $ty)),
                Some(Helper::Str(s)) => s
                    .parse::<$ty>()
                    .or_else(|_| s.parse::<f64>().map(|f| f as $ty))
                    .map(Some)
                    .map_err(serde::de::Error::custom),
                None => Ok(None),
            }
        }

        pub fn $vec_fn<'de, D>(deserializer: D) -> Result<Vec<$ty>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Helper {
                Int($ty),
                Float(f64),
                Str(String),
            }
            Vec::<Helper>::deserialize(deserializer)?
                .into_iter()
                .map(|h| match h {
                    Helper::Int(i) => Ok(i),
                    Helper::Float(f) => Ok(f as $ty),
                    Helper::Str(s) => s
                        .parse::<$ty>()
                        .or_else(|_| s.parse::<f64>().map(|f| f as $ty))
                        .map_err(serde::de::Error::custom),
                })
                .collect()
        }

        pub fn $opt_vec_fn<'de, D>(deserializer: D) -> Result<Option<Vec<$ty>>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(untagged)]
            enum Helper {
                Int($ty),
                Float(f64),
                Str(String),
            }
            match Option::<Vec<Helper>>::deserialize(deserializer)? {
                Some(v) => v
                    .into_iter()
                    .map(|h| match h {
                        Helper::Int(i) => Ok(i),
                        Helper::Float(f) => Ok(f as $ty),
                        Helper::Str(s) => s
                            .parse::<$ty>()
                            .or_else(|_| s.parse::<f64>().map(|f| f as $ty))
                            .map_err(serde::de::Error::custom),
                    })
                    .collect::<Result<Vec<$ty>, _>>()
                    .map(Some),
                None => Ok(None),
            }
        }
    };
}

impl_int_deserializers!(i8, deserialize_i8, deserialize_optional_i8, deserialize_vec_i8, deserialize_opt_vec_i8);
impl_int_deserializers!(i16, deserialize_i16, deserialize_optional_i16, deserialize_vec_i16, deserialize_opt_vec_i16);
impl_int_deserializers!(i32, deserialize_i32, deserialize_optional_i32, deserialize_vec_i32, deserialize_opt_vec_i32);
impl_int_deserializers!(i64, deserialize_i64, deserialize_optional_i64, deserialize_vec_i64, deserialize_opt_vec_i64);

"#,
        );

        if !self.required_helpers.is_empty() {
            out.push_str("// --- Helper Functions for Serde Version Attributes ---\n\n");
            for helper in &self.required_helpers {
                out.push_str(helper);
                out.push_str("\n\n");
            }
        }

        out.push_str("// --- Generated Data Structures ---\n\n");
        for code in self.generated_types.values() {
            out.push_str(code);
            out.push_str("\n\n");
        }

        out
    }

    fn generate_top_level_type(&mut self, name: &str, type_def: &TypeDef) {
        if self.value_overrides.contains(name) {
            self.generated_types.insert(
                name.to_string(),
                format!("pub type {} = serde_json::Value;", name),
            );
            return;
        }
        if self.generated_types.contains_key(name) || self.in_progress.contains(name) {
            return;
        }

        self.in_progress.insert(name.to_string());

        match type_def {
            TypeDef::Struct { fields, .. } => {
                if let Some((tag_key, registry)) = find_tagged_dispatcher(fields) {
                    let code = self.generate_dispatcher_enum_code(name, &tag_key, &registry);
                    self.generated_types.insert(name.to_string(), code);
                } else if is_map_struct(fields) {
                    let (key_type, val_type) = self.resolve_map_types(fields, name);
                    self.generated_types.insert(
                        name.to_string(),
                        format!("pub type {} = HashMap<{}, {}>;", name, key_type, val_type),
                    );
                } else {
                    let code = self.generate_struct_code(name, fields);
                    self.generated_types.insert(name.to_string(), code);
                }
            }
            TypeDef::Enum {
                enum_kind, values, ..
            } => {
                let code = if let Some((tag_key, registry)) = self.find_enum_dispatcher(name) {
                    self.generate_dispatcher_enum_code(name, &tag_key, &registry)
                } else {
                    self.generate_enum_code(name, enum_kind, values)
                };
                self.generated_types.insert(name.to_string(), code);
            }
            TypeDef::Template {
                child, type_params, ..
            } => {
                if let TypeDef::Struct { fields, .. } = child.as_ref() {
                    let params = type_params
                        .iter()
                        .map(|param| template_param_name(&param.path))
                        .collect::<Vec<_>>();
                    let previous_params =
                        std::mem::replace(&mut self.active_type_params, params.clone());
                    let mut code = self.generate_struct_code(name, fields);
                    self.active_type_params = previous_params;
                    let declaration = format!("pub struct {} {{", name);
                    let generic_declaration =
                        format!("pub struct {}<{}> {{", name, params.join(", "));
                    code = code.replacen(&declaration, &generic_declaration, 1);
                    self.generated_types.insert(name.to_string(), code);
                } else {
                    let rust_type = self.resolve_type(child, name);
                    self.generated_types.insert(
                        name.to_string(),
                        format!("pub type {} = {};", name, rust_type),
                    );
                }
            }
            TypeDef::Union { members, .. } => {
                let code = self.generate_union_code(name, members);
                self.generated_types.insert(name.to_string(), code);
            }
            other => {
                let rust_type = self.resolve_type(other, name);
                self.generated_types.insert(
                    name.to_string(),
                    format!("pub type {} = {};", name, rust_type),
                );
            }
        }

        self.in_progress.remove(name);
    }

    fn generate_struct_code(&mut self, struct_name: &str, fields: &[StructField]) -> String {
        let mut out = String::new();
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize)]\n");
        out.push_str(&format!("pub struct {} {{\n", struct_name));

        let mut visited_spreads = BTreeSet::new();
        let flattened_fields = self.collect_struct_fields_inner(fields, &mut visited_spreads);

        for (field_name, field_type, is_optional, attrs, desc) in flattened_fields {
            let (since, until) = extract_version_bounds(&attrs);
            if let Some(until_str) = &until {
                let ver = SemVer::parse(until_str);
                if ver <= MIN_VERSION {
                    continue;
                }
            }

            if let Some(identifier) = self.identifier_spec_for_type(&field_type) {
                write_identifier_comment(&mut out, "    ", &identifier);
            }

            if let Some(doc) = &desc {
                write_doc_comment(&mut out, "    ", doc);
            }

            let property_enum = field_name == "property"
                && matches!(&field_type, TypeDef::Reference { path, .. } if self.find_enum_dispatcher_path(path).is_some());
            let (serde_attr, final_type) = if property_enum {
                (
                    "#[serde(flatten)]".to_string(),
                    self.resolve_type(&field_type, struct_name),
                )
            } else {
                self.build_field_serde(
                    &field_name,
                    &field_type,
                    is_optional,
                    since.as_deref(),
                    until.as_deref(),
                    struct_name,
                )
            };

            if !serde_attr.is_empty() {
                out.push_str(&format!("    {}\n", serde_attr));
            }

            let (ident, rename) = sanitize_field_ident(&field_name);
            if !property_enum {
                if let Some(orig) = rename {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", orig));
                }
            }

            out.push_str(&format!("    pub {}: {},\n", ident, final_type));
        }

        for (index, field) in fields.iter().enumerate() {
            let StructField::Pair {
                key: FieldKey::Type(key_type),
                field_type,
                ..
            } = field
            else {
                continue;
            };
            let key_type =
                self.resolve_type(key_type, &format!("{}EntryKey{}", struct_name, index));
            let value_type =
                self.resolve_type(field_type, &format!("{}EntryValue{}", struct_name, index));
            out.push_str("    #[serde(flatten)]\n");
            out.push_str(&format!(
                "    pub entries_{}: HashMap<{}, {}>,\n",
                index, key_type, value_type
            ));
        }

        out.push('}');
        out
    }

    fn generate_enum_code(
        &mut self,
        enum_name: &str,
        _enum_kind: &str,
        values: &[EnumMember],
    ) -> String {
        let mut out = String::new();
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]\n");
        out.push_str(&format!("pub enum {} {{\n", enum_name));

        for val in values {
            let (since, until) = extract_version_bounds(&val.attributes);
            if let Some(since) = since {
                out.push_str(&format!("    /// Available since Minecraft {}.\n", since));
            }
            if let Some(until) = until {
                out.push_str(&format!("    /// Available until Minecraft {}.\n", until));
            }

            if let Some(desc) = &val.desc {
                write_doc_comment(&mut out, "    ", desc);
            }
            let variant_name = sanitize_type_ident(&val.identifier);

            match &val.value {
                EnumValue::String(s) => {
                    let (canonical, alias) = if let Some(stripped) = s.strip_prefix("minecraft:") {
                        (stripped.to_string(), Some(s.clone()))
                    } else if !s.contains(':') {
                        (s.clone(), Some(format!("minecraft:{}", s)))
                    } else {
                        (s.clone(), None)
                    };

                    if let Some(alias_str) = alias {
                        out.push_str(&format!(
                            "    #[serde(rename = \"{}\", alias = \"{}\")]\n",
                            canonical, alias_str
                        ));
                    } else {
                        out.push_str(&format!("    #[serde(rename = \"{}\")]\n", canonical));
                    }
                }
                EnumValue::Integer(i) => {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", i));
                }
                EnumValue::Float(f) => {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", f));
                }
                EnumValue::Boolean(b) => {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", b));
                }
            }

            out.push_str(&format!("    {},\n", variant_name));
        }

        out.push('}');
        out
    }

    fn generate_dispatcher_enum_code(
        &mut self,
        enum_name: &str,
        tag_key: &str,
        registry: &str,
    ) -> String {
        let mut out = String::new();
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize)]\n");
        out.push_str(&format!("#[serde(tag = \"{}\")]\n", tag_key));
        out.push_str(&format!("pub enum {} {{\n", enum_name));

        if let Some(variants) = self.mcdoc.mcdoc_dispatcher.get(registry) {
            let mut sorted_variants: Vec<_> = variants.iter().collect();
            sorted_variants.sort_by_key(|(k, _)| *k);
            let shared_fields = self.dispatcher_outer_fields(enum_name, tag_key, registry);

            for (variant_key, variant_type) in sorted_variants {
                if variant_key.starts_with('%') {
                    continue;
                }
                let (_, until) = match variant_type {
                    TypeDef::Reference { attributes, .. } => extract_version_bounds(attributes),
                    _ => (None, None),
                };
                if until.is_some_and(|version| SemVer::parse(&version) <= MIN_VERSION) {
                    continue;
                }

                let variant_name = sanitize_type_ident(variant_key);
                let (canonical, alias) =
                    if let Some(stripped) = variant_key.strip_prefix("minecraft:") {
                        (stripped.to_string(), Some(variant_key.clone()))
                    } else if !variant_key.contains(':') {
                        (
                            variant_key.clone(),
                            Some(format!("minecraft:{}", variant_key)),
                        )
                    } else {
                        (variant_key.clone(), None)
                    };

                if let Some(alias_str) = alias {
                    out.push_str(&format!(
                        "    #[serde(rename = \"{}\", alias = \"{}\")]\n",
                        canonical, alias_str
                    ));
                } else {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", canonical));
                }

                let resolved_def = match variant_type {
                    TypeDef::Reference { path, .. } => {
                        self.mcdoc.mcdoc.get(path).unwrap_or(variant_type)
                    }
                    _ => variant_type,
                };

                let mut visited_spreads = BTreeSet::new();
                let mut variant_fields = match resolved_def {
                    TypeDef::Struct { fields, .. } => {
                        self.collect_struct_fields_inner(fields, &mut visited_spreads)
                    }
                    _ => Vec::new(),
                };
                variant_fields.extend(shared_fields.iter().cloned());

                if variant_fields.is_empty() {
                    out.push_str(&format!("    {} {{}},\n", variant_name));
                } else {
                    out.push_str(&format!("    {} {{\n", variant_name));
                    for (field_name, field_type, is_optional, attrs, desc) in variant_fields {
                        let (since, until) = extract_version_bounds(&attrs);
                        if let Some(until_str) = &until {
                            if SemVer::parse(until_str) <= MIN_VERSION {
                                continue;
                            }
                        }

                        if let Some(identifier) = self.identifier_spec_for_type(&field_type) {
                            write_identifier_comment(&mut out, "        ", &identifier);
                        }

                        if let Some(doc) = &desc {
                            write_doc_comment(&mut out, "        ", doc);
                        }

                        let property_enum = field_name == "property"
                            && matches!(&field_type, TypeDef::Reference { path, .. } if self.find_enum_dispatcher_path(path).is_some());
                        let (serde_attr, mut final_type) = if property_enum {
                            (
                                "#[serde(flatten)]".to_string(),
                                self.resolve_type(
                                    &field_type,
                                    &format!("{}_{}", enum_name, variant_name),
                                ),
                            )
                        } else {
                            self.build_field_serde(
                                &field_name,
                                &field_type,
                                is_optional,
                                since.as_deref(),
                                until.as_deref(),
                                &format!("{}_{}", enum_name, variant_name),
                            )
                        };

                        if final_type == enum_name {
                            final_type = "Box<Self>".to_string();
                        } else if final_type == format!("Option<{}>", enum_name) {
                            final_type = "Option<Box<Self>>".to_string();
                        } else if final_type == format!("Vec<{}>", enum_name) {
                            final_type = "Vec<Self>".to_string();
                        }

                        if !serde_attr.is_empty() {
                            out.push_str(&format!("        {}\n", serde_attr));
                        }

                        let (ident, rename) = sanitize_field_ident(&field_name);
                        if !property_enum {
                            if let Some(orig) = rename {
                                out.push_str(&format!("        #[serde(rename = \"{}\")]\n", orig));
                            }
                        }

                        out.push_str(&format!("        {}: {},\n", ident, final_type));
                    }
                    out.push_str("    },\n");
                }
            }
        }

        // Some registries omit tags whose payload is empty (for example,
        // `item_model`'s `empty` variant). The tag enum remains the complete
        // list of accepted values, so include any tags not represented by the
        // dispatcher as unit variants.
        let tag_enum_values = self
            .mcdoc
            .mcdoc
            .iter()
            .find(|(path, _)| type_name_from_path(path) == enum_name)
            .and_then(|(_, type_def)| match type_def {
                TypeDef::Enum { values, .. } => Some(values),
                TypeDef::Struct { fields, .. } => fields
                    .iter()
                    .find_map(|field| match field {
                        StructField::Pair {
                            key: FieldKey::String(key),
                            field_type,
                            ..
                        } if key == tag_key => match field_type {
                            TypeDef::Reference { path, .. } => self.mcdoc.mcdoc.get(path),
                            other => Some(other),
                        },
                        _ => None,
                    })
                    .and_then(|type_def| match type_def {
                        TypeDef::Enum { values, .. } => Some(values),
                        _ => None,
                    }),
                _ => None,
            });

        if let Some(values) = tag_enum_values {
            let dispatcher_values = self.mcdoc.mcdoc_dispatcher.get(registry);
            for value in values {
                let EnumValue::String(tag) = &value.value else {
                    continue;
                };
                let (_, until) = extract_version_bounds(&value.attributes);
                if until.is_some_and(|version| SemVer::parse(&version) <= MIN_VERSION) {
                    continue;
                }
                if dispatcher_values.is_some_and(|variants| variants.contains_key(tag)) {
                    continue;
                }

                let variant_name = sanitize_type_ident(&value.identifier);
                let (canonical, alias) = if let Some(stripped) = tag.strip_prefix("minecraft:") {
                    (stripped.to_string(), Some(tag.clone()))
                } else if !tag.contains(':') {
                    (tag.clone(), Some(format!("minecraft:{}", tag)))
                } else {
                    (tag.clone(), None)
                };
                if let Some(alias_str) = alias {
                    out.push_str(&format!(
                        "    #[serde(rename = \"{}\", alias = \"{}\")]\n",
                        canonical, alias_str
                    ));
                } else {
                    out.push_str(&format!("    #[serde(rename = \"{}\")]\n", canonical));
                }
                out.push_str(&format!("    {} {{}},\n", variant_name));
            }
        }

        out.push('}');
        out
    }

    fn generate_union_code(&mut self, union_name: &str, members: &[TypeDef]) -> String {
        let mut out = String::new();
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize)]\n#[serde(untagged)]\n");
        let generic_params = if self.active_type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", self.active_type_params.join(", "))
        };
        out.push_str(&format!("pub enum {}{} {{\n", union_name, generic_params));
        for (index, member) in members.iter().enumerate() {
            let variant = union_variant_name(member, index);
            let ty = self.resolve_type(member, &format!("{}{}", union_name, variant));
            if let Some(identifier) = self.identifier_spec_for_type(member) {
                write_identifier_comment(&mut out, "    ", &identifier);
            }
            let (since, until) = type_version_bounds(member);
            if let Some(since) = since {
                out.push_str(&format!("    /// Available since Minecraft {}.\n", since));
            }
            if let Some(until) = until {
                out.push_str(&format!("    /// Available until Minecraft {}.\n", until));
            }
            out.push_str(&format!("    {}({}),\n", variant, ty));
        }
        out.push('}');
        out
    }

    fn generate_dispatcher_value_code(&mut self, type_name: &str, registry: &str) -> String {
        if self.generated_types.contains_key(type_name) {
            return type_name.to_string();
        }

        // Install a placeholder before resolving payloads so recursive
        // registries can refer back to this type safely.
        self.generated_types
            .insert(type_name.to_string(), String::new());
        let mut out = String::new();
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize)]\n#[serde(untagged)]\n");
        out.push_str(&format!("pub enum {} {{\n", type_name));

        if let Some(variants) = self.mcdoc.mcdoc_dispatcher.get(registry) {
            let mut sorted_variants: Vec<_> = variants.iter().collect();
            sorted_variants.sort_by_key(|(key, _)| *key);
            for (key, payload_type) in sorted_variants {
                if key.starts_with('%')
                    || matches!(payload_type, TypeDef::Union { members, .. } if members.is_empty())
                {
                    continue;
                }

                let variant_name = sanitize_type_ident(key);
                if let Some(identifier) = self.identifier_spec_for_type(payload_type) {
                    write_identifier_comment(&mut out, "    ", &identifier);
                }
                let payload_type =
                    self.resolve_type(payload_type, &format!("{}{}", type_name, variant_name));
                out.push_str(&format!("    {}({}),\n", variant_name, payload_type));
            }
        }

        out.push('}');
        self.generated_types
            .insert(type_name.to_string(), out.clone());
        type_name.to_string()
    }

    fn collect_struct_fields_inner(
        &self,
        fields: &[StructField],
        visited: &mut BTreeSet<String>,
    ) -> Vec<(String, TypeDef, bool, Vec<Attribute>, Option<String>)> {
        let mut result = Vec::new();
        let embedded_dispatcher = self.find_embedded_dispatcher(fields);

        for f in fields {
            match f {
                StructField::Pair {
                    key,
                    field_type,
                    optional,
                    desc,
                    attributes,
                } => {
                    let FieldKey::String(key_name) = key else {
                        continue;
                    };
                    result.push((
                        key_name.clone(),
                        field_type.clone(),
                        *optional,
                        attributes.clone(),
                        desc.clone(),
                    ));
                }
                StructField::Spread {
                    field_type,
                    attributes,
                    desc,
                } => {
                    let (_, until) = extract_version_bounds(attributes);
                    if let Some(until_str) = until
                        && SemVer::parse(&until_str) <= MIN_VERSION
                    {
                        continue;
                    }

                    if let TypeDef::Reference { path, .. } = field_type {
                        if !visited.insert(path.clone()) {
                            continue;
                        }

                        if let Some(TypeDef::Struct {
                            fields: inner_fields,
                            ..
                        }) = self.mcdoc.mcdoc.get(path)
                        {
                            let mut inlined =
                                self.collect_struct_fields_inner(inner_fields, visited);
                            for item in &mut inlined {
                                if item.3.is_empty() {
                                    item.3 = attributes.clone();
                                }
                                if item.4.is_none() {
                                    item.4 = desc.clone();
                                }
                            }
                            result.extend(inlined);
                        }
                    } else if let TypeDef::Struct {
                        fields: inner_fields,
                        ..
                    } = field_type
                    {
                        result.extend(self.collect_struct_fields_inner(inner_fields, visited));
                    } else if let TypeDef::Concrete {
                        child, type_args, ..
                    } = field_type
                        && let TypeDef::Reference { path, .. } = child.as_ref()
                        && visited.insert(path.clone())
                        && let Some(TypeDef::Template {
                            child: template_child,
                            type_params,
                            ..
                        }) = self.mcdoc.mcdoc.get(path)
                        && let TypeDef::Struct {
                            fields: template_fields,
                            ..
                        } = template_child.as_ref()
                    {
                        let substitutions = type_params
                            .iter()
                            .map(|param| param.path.clone())
                            .zip(type_args.iter().cloned())
                            .collect();
                        let fields = substitute_template_fields(template_fields, &substitutions);
                        result.extend(self.collect_struct_fields_inner(&fields, visited));
                    } else if let TypeDef::Dispatcher { registry, .. } = field_type {
                        if embedded_dispatcher
                            .as_ref()
                            .is_some_and(|(_, _, embedded_registry)| embedded_registry == registry)
                        {
                            continue;
                        }
                    }
                }
            }
        }

        let mut deduplicated: Vec<(String, TypeDef, bool, Vec<Attribute>, Option<String>)> =
            Vec::with_capacity(result.len());
        for mut field in result {
            if let Some(existing) = deduplicated.iter_mut().find(|item| item.0 == field.0) {
                let has_version_bounds = extract_version_bounds(&existing.3).0.is_some()
                    || extract_version_bounds(&existing.3).1.is_some()
                    || extract_version_bounds(&field.3).0.is_some()
                    || extract_version_bounds(&field.3).1.is_some();
                existing.2 |= field.2 || has_version_bounds;

                // When adjacent schema versions define the same field, keep
                // the newer definition and make the merged field optional.
                if extract_version_bounds(&field.3).0.is_some() {
                    existing.1 = field.1;
                }
                if has_version_bounds {
                    existing.3.retain(|attr| {
                        !matches!(attr, Attribute::Since { .. } | Attribute::Until { .. })
                    });
                    field.3.retain(|attr| {
                        !matches!(attr, Attribute::Since { .. } | Attribute::Until { .. })
                    });
                }
                existing.3.extend(field.3);
                if existing.4.is_none() {
                    existing.4 = field.4;
                }
            } else {
                deduplicated.push(field);
            }
        }

        deduplicated
    }

    fn dispatcher_outer_fields(
        &self,
        enum_name: &str,
        tag_key: &str,
        registry: &str,
    ) -> Vec<(String, TypeDef, bool, Vec<Attribute>, Option<String>)> {
        let Some((_, type_def)) = self
            .mcdoc
            .mcdoc
            .iter()
            .find(|(path, _)| type_name_from_path(path) == enum_name)
        else {
            return Vec::new();
        };
        let TypeDef::Struct { fields, .. } = type_def else {
            return Vec::new();
        };
        let has_dispatcher = fields.iter().any(|field| match field {
            StructField::Spread {
                field_type:
                    TypeDef::Dispatcher {
                        parallel_indices,
                        registry: field_registry,
                        ..
                    },
                ..
            } if field_registry == registry => parallel_indices.iter().any(|index| match index {
                IndexAccessor::Dynamic { accessor } => accessor
                    .iter()
                    .any(|token| matches!(token, AccessorToken::String(key) if key == tag_key)),
                _ => false,
            }),
            _ => false,
        });
        if !has_dispatcher {
            return Vec::new();
        }

        fields
            .iter()
            .filter_map(|field| match field {
                StructField::Pair {
                    key: FieldKey::String(key),
                    field_type,
                    optional,
                    attributes,
                    desc,
                } if key != tag_key => Some((
                    key.clone(),
                    field_type.clone(),
                    *optional,
                    attributes.clone(),
                    desc.clone(),
                )),
                _ => None,
            })
            .collect()
    }

    fn find_enum_dispatcher(&self, enum_name: &str) -> Option<(String, String)> {
        let enum_path = self
            .mcdoc
            .mcdoc
            .keys()
            .find(|path| type_name_from_path(path) == enum_name)?;
        let (tag_key, _, registry) = self.find_enum_dispatcher_path(enum_path)?;
        Some((tag_key, registry))
    }

    fn find_enum_dispatcher_path(&self, enum_path: &str) -> Option<(String, String, String)> {
        for type_def in self.mcdoc.mcdoc.values() {
            let TypeDef::Struct { fields, .. } = type_def else {
                continue;
            };
            if let Some((tag_key, field_path, registry)) = self.find_embedded_dispatcher(fields)
                && field_path == enum_path
            {
                return Some((tag_key, field_path, registry));
            }
        }
        None
    }

    fn find_embedded_dispatcher(&self, fields: &[StructField]) -> Option<(String, String, String)> {
        let tag_field = fields.iter().find_map(|field| match field {
            StructField::Pair {
                key: FieldKey::String(key),
                field_type: TypeDef::Reference { path, .. },
                ..
            } => Some((key.clone(), path.clone())),
            _ => None,
        })?;
        let registry = fields.iter().find_map(|field| match field {
            StructField::Spread {
                field_type:
                    TypeDef::Dispatcher {
                        parallel_indices,
                        registry,
                        ..
                    },
                ..
            } if parallel_indices.iter().any(|index| match index {
                IndexAccessor::Dynamic { accessor } => accessor.iter().any(
                    |token| matches!(token, AccessorToken::String(key) if key == &tag_field.0),
                ),
                _ => false,
            }) =>
            {
                Some(registry.clone())
            }
            _ => None,
        })?;
        Some((tag_field.0, tag_field.1, registry))
    }

    fn resolve_type(&mut self, type_def: &TypeDef, context_name: &str) -> String {
        if self
            .value_overrides
            .contains(&sanitize_type_ident(context_name))
        {
            return "serde_json::Value".to_string();
        }

        match type_def {
            TypeDef::Boolean { .. } => "bool".to_string(),
            TypeDef::String { attributes, .. } => self
                .identifier_type(attributes)
                .unwrap_or("String")
                .to_string(),
            TypeDef::Float { .. } => "f32".to_string(),
            TypeDef::Double { .. } => "f64".to_string(),
            TypeDef::Byte { .. } => "i8".to_string(),
            TypeDef::Short { .. } => "i16".to_string(),
            TypeDef::Long { .. } => "i64".to_string(),
            TypeDef::Int { value_range, .. } => {
                if let Some(r) = value_range {
                    let min = r.min.unwrap_or(0.0);
                    let max = r.max.unwrap_or(i32::MAX as f64);
                    if min >= 0.0 && max <= 255.0 {
                        return "u8".to_string();
                    } else if min >= 0.0 && max <= 65535.0 {
                        return "u16".to_string();
                    }
                }
                "i32".to_string()
            }
            TypeDef::IntArray { length_range, .. } => {
                if let Some(r) = length_range
                    && let (Some(min), Some(max)) = (r.min, r.max)
                    && min == max
                {
                    return format!("[i32; {}]", min as usize);
                }
                "Vec<i32>".to_string()
            }
            TypeDef::List {
                item, length_range, ..
            } => {
                let inner = self.resolve_type(item, context_name);
                if let Some(r) = length_range
                    && let (Some(min), Some(max)) = (r.min, r.max)
                    && min == max
                {
                    return format!("[{}; {}]", inner, min as usize);
                }
                format!("Vec<{}>", inner)
            }
            TypeDef::Tuple { items, .. } => {
                let parts: Vec<_> = items
                    .iter()
                    .map(|it| self.resolve_type(it, context_name))
                    .collect();
                format!("({})", parts.join(", "))
            }
            TypeDef::Reference { path, .. } => {
                if self.is_template_parameter(path) {
                    return template_param_name(path);
                }
                if let Some(spec) = self.identifier_spec_for_type(type_def)
                    && self.mcdoc.mcdoc.get(path).is_some_and(is_string_type)
                {
                    return self.identifier_rust_type(&spec).to_string();
                }
                let rust_name = type_name_from_path(path);
                if let Some(target) = self.mcdoc.mcdoc.get(path).cloned() {
                    let previous_params = std::mem::take(&mut self.active_type_params);
                    self.generate_top_level_type(&rust_name, &target);
                    self.active_type_params = previous_params;
                }
                rust_name
            }
            TypeDef::Struct { fields, .. } => {
                if is_map_struct(fields) {
                    let (key_type, val_type) = self.resolve_map_types(fields, context_name);
                    format!("HashMap<{}, {}>", key_type, val_type)
                } else {
                    let inline_name = format!("{}Anonymous", sanitize_type_ident(context_name));
                    let code = self.generate_struct_code(&inline_name, fields);
                    self.generated_types.insert(inline_name.clone(), code);
                    inline_name
                }
            }
            TypeDef::Union { members, .. } => self.resolve_union(members, context_name),
            TypeDef::Concrete {
                child, type_args, ..
            } => {
                if let TypeDef::Reference { path, .. } = child.as_ref()
                    && let Some(TypeDef::Template { type_params, .. }) = self.mcdoc.mcdoc.get(path)
                {
                    let rust_name = type_name_from_path(path);
                    if let Some(target) = self.mcdoc.mcdoc.get(path).cloned() {
                        self.generate_top_level_type(&rust_name, &target);
                    }
                    let args = type_args
                        .iter()
                        .map(|arg| self.resolve_type(arg, context_name))
                        .collect::<Vec<_>>();
                    if args.len() == type_params.len() {
                        return format!("{}<{}>", rust_name, args.join(", "));
                    }
                }
                self.resolve_type(child, context_name)
            }
            TypeDef::Enum {
                enum_kind, values, ..
            } => {
                let inline_name = sanitize_type_ident(context_name);
                let code = self.generate_enum_code(&inline_name, enum_kind, values);
                self.generated_types.insert(inline_name.clone(), code);
                inline_name
            }
            TypeDef::Template { child, .. } => self.resolve_type(child, context_name),
            TypeDef::Indexed { child, .. } => self.resolve_type(child, context_name),
            TypeDef::Literal { value, .. } => match value {
                LiteralValue::String(_) => "String".to_string(),
                LiteralValue::Boolean(_) => "bool".to_string(),
                LiteralValue::Int(_) => "i32".to_string(),
                LiteralValue::Float(_) => "f32".to_string(),
                LiteralValue::Double(_) => "f64".to_string(),
                LiteralValue::Byte(_) => "i8".to_string(),
                LiteralValue::Short(_) => "i16".to_string(),
                LiteralValue::Long(_) => "i64".to_string(),
            },
            TypeDef::ByteArray { length_range, .. } => {
                if let Some(r) = length_range
                    && let (Some(min), Some(max)) = (r.min, r.max)
                    && min == max
                {
                    return format!("[i8; {}]", min as usize);
                }
                "Vec<i8>".to_string()
            }
            TypeDef::LongArray { length_range, .. } => {
                if let Some(r) = length_range
                    && let (Some(min), Some(max)) = (r.min, r.max)
                    && min == max
                {
                    return format!("[i64; {}]", min as usize);
                }
                "Vec<i64>".to_string()
            }
            TypeDef::Dispatcher { registry, .. } => {
                self.generate_dispatcher_value_code(&sanitize_type_ident(context_name), registry)
            }
            TypeDef::Any { .. } => "serde_json::Value".to_string(),
        }
    }

    fn resolve_map_types(
        &mut self,
        fields: &[StructField],
        context_name: &str,
    ) -> (String, String) {
        let Some(StructField::Pair {
            key: FieldKey::Type(key_type),
            field_type: value_type,
            ..
        }) = fields.first()
        else {
            return ("String".to_string(), "serde_json::Value".to_string());
        };

        let key_type = self.resolve_type(key_type, &format!("{}Key", context_name));
        let value_type = self.resolve_type(value_type, context_name);
        (key_type, value_type)
    }

    fn is_template_parameter(&self, path: &str) -> bool {
        self.mcdoc.mcdoc.values().any(|type_def| {
            matches!(type_def, TypeDef::Template { type_params, .. }
                if type_params.iter().any(|param| param.path == path))
        })
    }

    fn identifier_spec_for_type(&self, type_def: &TypeDef) -> Option<IdentifierSpec> {
        fn find(
            generator: &Generator<'_>,
            type_def: &TypeDef,
            visited: &mut BTreeSet<String>,
        ) -> Option<IdentifierSpec> {
            match type_def {
                TypeDef::String { attributes, .. } => identifier_spec(attributes),
                TypeDef::Reference {
                    path, attributes, ..
                } => {
                    if !visited.insert(path.clone()) {
                        return None;
                    }
                    let target = generator.mcdoc.mcdoc.get(path)?;
                    if is_string_type(target)
                        && let Some(spec) = identifier_spec(attributes)
                    {
                        return Some(spec);
                    }
                    find(generator, target, visited)
                }
                TypeDef::List { item, .. } => find(generator, item, visited),
                TypeDef::Concrete {
                    child, type_args, ..
                } => find(generator, child, visited).or_else(|| {
                    type_args
                        .iter()
                        .find_map(|argument| find(generator, argument, visited))
                }),
                TypeDef::Template { child, .. } | TypeDef::Indexed { child, .. } => {
                    find(generator, child, visited)
                }
                _ => None,
            }
        }

        find(self, type_def, &mut BTreeSet::new())
    }

    fn identifier_type(&self, attributes: &[Attribute]) -> Option<&'static str> {
        if is_texture_value_slot(attributes) {
            return Some("crate::resource_pack::identifier::TextureId");
        }

        identifier_spec(attributes).map(|spec| self.identifier_rust_type(&spec))
    }

    fn identifier_rust_type(&self, identifier: &IdentifierSpec) -> &'static str {
        if self.use_specialized_identifier_types {
            identifier_rust_type(identifier.registry.as_deref())
        } else {
            "crate::resource_pack::identifier::Identifier"
        }
    }

    fn resolve_union(&mut self, members: &[TypeDef], context_name: &str) -> String {
        if let Some(float_type) = floating_union_type(members) {
            return float_type.to_string();
        }

        let all_ints = members.iter().all(|m| {
            matches!(
                m,
                TypeDef::Literal {
                    value: LiteralValue::Int(_),
                    ..
                } | TypeDef::Int { .. }
            )
        });
        if all_ints {
            return "i32".to_string();
        }

        let all_strings = members.iter().all(|m| {
            matches!(
                m,
                TypeDef::Literal {
                    value: LiteralValue::String(_),
                    ..
                } | TypeDef::String { .. }
            )
        });
        if all_strings {
            return "String".to_string();
        }

        let union_name = sanitize_type_ident(context_name);
        if !self.generated_types.contains_key(&union_name) {
            self.generated_types
                .insert(union_name.clone(), String::new());
            let code = self.generate_union_code(&union_name, members);
            self.generated_types.insert(union_name.clone(), code);
        }
        if self.active_type_params.is_empty() {
            union_name
        } else {
            format!("{}<{}>", union_name, self.active_type_params.join(", "))
        }
    }

    fn build_field_serde(
        &mut self,
        field_name: &str,
        field_type: &TypeDef,
        is_optional: bool,
        since: Option<&str>,
        until: Option<&str>,
        context: &str,
    ) -> (String, String) {
        let raw_type = self.resolve_type(
            field_type,
            &format!("{}{}", context, sanitize_type_ident(field_name)),
        );
        let is_opt = is_optional || since.is_some() || until.is_some();
        let final_type = if is_opt {
            format!("Option<{}>", raw_type)
        } else {
            raw_type.clone()
        };

        let mut attrs = Vec::new();

        for int_type in &["i8", "i16", "i32", "i64"] {
            if raw_type == *int_type {
                if is_opt {
                    attrs.push(format!(
                        "deserialize_with = \"deserialize_optional_{}\"",
                        int_type
                    ));
                } else {
                    attrs.push(format!("deserialize_with = \"deserialize_{}\"", int_type));
                }
                break;
            } else if raw_type == format!("Vec<{}>", int_type) {
                if is_opt {
                    attrs.push(format!(
                        "deserialize_with = \"deserialize_opt_vec_{}\"",
                        int_type
                    ));
                } else {
                    attrs.push(format!(
                        "deserialize_with = \"deserialize_vec_{}\"",
                        int_type
                    ));
                }
                break;
            }
        }

        if is_opt {
            let skip_func = self.resolve_skip_serializing_fn(since, until);
            attrs.push(format!("skip_serializing_if = \"{}\"", skip_func));
        }

        let attr_str = if attrs.is_empty() {
            String::new()
        } else {
            format!("#[serde({})]", attrs.join(", "))
        };

        (attr_str, final_type)
    }

    fn resolve_skip_serializing_fn(&mut self, since: Option<&str>, until: Option<&str>) -> String {
        let since_suffix = since.and_then(|s| {
            let ver = SemVer::parse(s);
            if ver > MIN_VERSION {
                find_matching_milestone(ver)
            } else {
                None
            }
        });

        let until_suffix = until.and_then(|u| {
            let ver = SemVer::parse(u);
            if ver > MIN_VERSION {
                find_previous_milestone(ver)
            } else {
                None
            }
        });

        match (since_suffix, until_suffix) {
            (Some(s), Some(u)) => {
                let fn_name = format!("is_none_or_outside_{}_and_{}", s, u);
                self.required_helpers.insert(format!(
                    "#[inline]\npub fn {}<T>(val: &Option<T>) -> bool {{\n\
                     \x20   val.is_none() || crate::version::is_older_than_{}(()) || crate::version::is_newer_than_{}(())\n\
                     }}",
                    fn_name, s, u
                ));
                fn_name
            }
            (Some(s), None) => {
                let fn_name = format!("is_none_or_older_than_{}", s);
                self.required_helpers.insert(format!(
                    "#[inline]\npub fn {}<T>(val: &Option<T>) -> bool {{\n\
                     \x20   val.is_none() || crate::version::is_older_than_{}(())\n\
                     }}",
                    fn_name, s
                ));
                fn_name
            }
            (None, Some(u)) => {
                let fn_name = format!("is_none_or_newer_than_{}", u);
                self.required_helpers.insert(format!(
                    "#[inline]\npub fn {}<T>(val: &Option<T>) -> bool {{\n\
                     \x20   val.is_none() || crate::version::is_newer_than_{}(())\n\
                     }}",
                    fn_name, u
                ));
                fn_name
            }
            (None, None) => "Option::is_none".to_string(),
        }
    }
}

// =========================================================================
// Utilities & Helpers
// =========================================================================

fn is_map_struct(fields: &[StructField]) -> bool {
    fields.len() == 1
        && matches!(
            &fields[0],
            StructField::Pair {
                key: FieldKey::Type(_),
                ..
            }
        )
}

fn template_param_name(path: &str) -> String {
    path.rsplit("::")
        .find(|part| !part.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| type_name_from_path(path))
}

#[derive(Debug, Clone)]
struct IdentifierSpec {
    registry: Option<String>,
}

fn write_identifier_comment(out: &mut String, indent: &str, identifier: &IdentifierSpec) {
    match identifier.registry.as_deref() {
        Some(registry) => out.push_str(&format!(
            "{}/// Identifier for the `{}` registry.\n",
            indent, registry
        )),
        None => out.push_str(&format!("{}/// Identifier.\n", indent)),
    }
}

fn identifier_spec(attributes: &[Attribute]) -> Option<IdentifierSpec> {
    attributes.iter().find_map(|attribute| {
        let Attribute::Id { value } = attribute else {
            return None;
        };
        let registry = value.as_ref().and_then(|value| match value {
            LiteralOrTreeAttr::Literal {
                value: LiteralValue::String(registry),
            } => Some(registry.clone()),
            LiteralOrTreeAttr::Literal { .. } => None,
            LiteralOrTreeAttr::Tree { values } => {
                values.get("registry").and_then(|value| match value {
                    AttrValue::Literal {
                        value: LiteralValue::String(registry),
                    } => Some(registry.clone()),
                    _ => None,
                })
            }
        });
        Some(IdentifierSpec { registry })
    })
}

fn is_texture_value_slot(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        let Attribute::TextureSlot { value } = attribute else {
            return false;
        };
        let TreeAttr::Tree { values } = value;
        matches!(
            values.get("kind"),
            Some(AttrValue::Literal {
                value: LiteralValue::String(kind)
            }) if kind == "value"
        )
    })
}

fn identifier_rust_type(registry: Option<&str>) -> &'static str {
    match registry {
        Some("texture") => "crate::resource_pack::identifier::TextureId",
        Some("model") => "crate::resource_pack::identifier::ModelId",
        Some("sound") => "crate::resource_pack::identifier::SoundId",
        _ => "crate::resource_pack::identifier::Identifier",
    }
}

fn is_string_type(type_def: &TypeDef) -> bool {
    matches!(type_def, TypeDef::String { .. })
}

fn collect_generated_type_dependencies(
    name: &str,
    generated_types: &BTreeMap<String, String>,
    type_names: &BTreeSet<String>,
    reachable: &mut BTreeSet<String>,
) {
    if !reachable.insert(name.to_string()) {
        return;
    }

    let Some(code) = generated_types.get(name) else {
        return;
    };
    for token in code.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
        if type_names.contains(token) && !reachable.contains(token) {
            collect_generated_type_dependencies(token, generated_types, type_names, reachable);
        }
    }
}

fn substitute_template_fields(
    fields: &[StructField],
    substitutions: &BTreeMap<String, TypeDef>,
) -> Vec<StructField> {
    fields
        .iter()
        .cloned()
        .map(|mut field| {
            match &mut field {
                StructField::Pair {
                    key, field_type, ..
                } => {
                    if let FieldKey::Type(key_type) = key {
                        **key_type = substitute_template_type(key_type, substitutions);
                    }
                    *field_type = substitute_template_type(field_type, substitutions);
                }
                StructField::Spread { field_type, .. } => {
                    *field_type = substitute_template_type(field_type, substitutions);
                }
            }
            field
        })
        .collect()
}

fn substitute_template_type(
    type_def: &TypeDef,
    substitutions: &BTreeMap<String, TypeDef>,
) -> TypeDef {
    if let TypeDef::Reference { path, .. } = type_def
        && let Some(replacement) = substitutions.get(path)
    {
        return replacement.clone();
    }

    let mut result = type_def.clone();
    match &mut result {
        TypeDef::Struct { fields, .. } => {
            *fields = substitute_template_fields(fields, substitutions);
        }
        TypeDef::Union { members, .. } => {
            *members = members
                .iter()
                .map(|member| substitute_template_type(member, substitutions))
                .collect();
        }
        TypeDef::List { item, .. } => {
            **item = substitute_template_type(item, substitutions);
        }
        TypeDef::Tuple { items, .. } => {
            *items = items
                .iter()
                .map(|item| substitute_template_type(item, substitutions))
                .collect();
        }
        TypeDef::Concrete {
            child, type_args, ..
        } => {
            **child = substitute_template_type(child, substitutions);
            *type_args = type_args
                .iter()
                .map(|arg| substitute_template_type(arg, substitutions))
                .collect();
        }
        TypeDef::Template { child, .. } | TypeDef::Indexed { child, .. } => {
            **child = substitute_template_type(child, substitutions);
        }
        _ => {}
    }
    result
}

fn floating_union_type(members: &[TypeDef]) -> Option<&'static str> {
    fn inspect(type_def: &TypeDef, has_float: &mut bool, has_double: &mut bool) -> bool {
        match type_def {
            TypeDef::Float { .. }
            | TypeDef::Literal {
                value: LiteralValue::Float(_),
                ..
            } => {
                *has_float = true;
                true
            }
            TypeDef::Double { .. } => {
                *has_double = true;
                true
            }
            TypeDef::Literal {
                value: LiteralValue::Double(_),
                ..
            } => {
                *has_double = true;
                true
            }
            TypeDef::Union { members, .. } => members
                .iter()
                .all(|member| inspect(member, has_float, has_double)),
            _ => false,
        }
    }

    let mut has_float = false;
    let mut has_double = false;
    if !members
        .iter()
        .all(|member| inspect(member, &mut has_float, &mut has_double))
    {
        return None;
    }

    Some(if has_double && !has_float {
        "f64"
    } else {
        "f32"
    })
}

fn union_variant_name(member: &TypeDef, index: usize) -> String {
    let base = match member {
        TypeDef::Reference { path, .. } => type_name_from_path(path),
        TypeDef::Struct { .. } => format!("Object{}", index + 1),
        TypeDef::List { .. } => format!("List{}", index + 1),
        TypeDef::Tuple { .. } => format!("Tuple{}", index + 1),
        TypeDef::String { .. }
        | TypeDef::Literal {
            value: LiteralValue::String(_),
            ..
        } => "String".to_string(),
        TypeDef::Boolean { .. }
        | TypeDef::Literal {
            value: LiteralValue::Boolean(_),
            ..
        } => "Boolean".to_string(),
        TypeDef::Int { .. }
        | TypeDef::Literal {
            value: LiteralValue::Int(_),
            ..
        } => "Integer".to_string(),
        TypeDef::Float { .. }
        | TypeDef::Literal {
            value: LiteralValue::Float(_),
            ..
        } => "Float".to_string(),
        TypeDef::Double { .. }
        | TypeDef::Literal {
            value: LiteralValue::Double(_),
            ..
        } => "Double".to_string(),
        _ => format!("Variant{}", index + 1),
    };
    format!("{}{}", base, index + 1)
}

fn extract_version_bounds(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let mut since = None;
    let mut until = None;

    for attr in attrs {
        match attr {
            Attribute::Since { value } => {
                if let LiteralValue::String(s) = value.value() {
                    since = Some(s.clone());
                }
            }
            Attribute::Until { value } => {
                if let LiteralValue::String(s) = value.value() {
                    until = Some(s.clone());
                }
            }
            _ => {}
        }
    }

    (since, until)
}

fn type_version_bounds(type_def: &TypeDef) -> (Option<String>, Option<String>) {
    match type_def {
        TypeDef::Reference { attributes, .. } => extract_version_bounds(attributes),
        _ => (None, None),
    }
}

fn find_matching_milestone(ver: SemVer) -> Option<&'static str> {
    for (v_str, suffix) in SUPPORTED_MILESTONES.iter().rev() {
        if ver >= SemVer::parse(v_str) {
            return Some(suffix);
        }
    }
    None
}

fn find_previous_milestone(ver: SemVer) -> Option<&'static str> {
    for (v_str, suffix) in SUPPORTED_MILESTONES.iter().rev() {
        if SemVer::parse(v_str) < ver {
            return Some(suffix);
        }
    }
    None
}

pub fn type_name_from_path(path: &str) -> String {
    let trimmed = path.trim_start_matches(':');
    let mut parts = Vec::new();

    for segment in trimmed.split("::") {
        if segment.is_empty() {
            continue;
        }

        if segment.starts_with('<') && segment.ends_with('>') {
            let inner = segment.trim_matches(|c| c == '<' || c == '>');
            let sanitized: String = inner
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            parts.push(sanitized);
        } else {
            let sanitized: String = segment
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            parts.push(sanitized);
        }
    }

    let mut result = parts.join("_");

    if result.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        result.insert(0, '_');
    }

    result
}

fn sanitize_type_ident(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            if capitalize {
                out.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                out.push(c);
            }
        } else {
            capitalize = true;
        }
    }
    out
}

fn sanitize_field_ident(raw: &str) -> (String, Option<String>) {
    let is_keyword = matches!(
        raw,
        "type"
            | "match"
            | "ref"
            | "fn"
            | "move"
            | "in"
            | "loop"
            | "for"
            | "struct"
            | "enum"
            | "as"
            | "use"
            | "mod"
            | "where"
            | "crate"
            | "self"
            | "super"
            | "pub"
            | "const"
            | "static"
            | "true"
            | "false"
    );

    if is_keyword {
        (format!("r#{}", raw), Some(raw.to_string()))
    } else if raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        (raw.to_string(), None)
    } else {
        let mut clean: String = raw
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        if clean.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            clean.insert(0, '_');
        }
        (clean, Some(raw.to_string()))
    }
}

fn write_doc_comment(out: &mut String, indent: &str, doc: &str) {
    let normalized = doc.replace("\r\n", "\n").replace("\\n", "\n");
    for line in normalized.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            out.push_str(&format!("{}///\n", indent));
        } else {
            out.push_str(&format!("{}/// {}\n", indent, trimmed));
        }
    }
}

fn find_tagged_dispatcher(fields: &[StructField]) -> Option<(String, String)> {
    let mut tag_key = None;
    let mut registry_name = None;

    for field in fields {
        match field {
            StructField::Pair {
                key: FieldKey::String(k),
                ..
            } => {
                if k == "type" {
                    tag_key = Some(k.clone());
                }
            }
            StructField::Spread {
                field_type:
                    TypeDef::Dispatcher {
                        parallel_indices,
                        registry,
                        ..
                    },
                ..
            } => {
                for idx in parallel_indices {
                    if let IndexAccessor::Dynamic { accessor } = idx {
                        if accessor
                            .iter()
                            .any(|token| matches!(token, AccessorToken::String(s) if s == "type"))
                        {
                            registry_name = Some(registry.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let (Some(k), Some(reg)) = (tag_key, registry_name) {
        Some((k, reg))
    } else {
        None
    }
}
