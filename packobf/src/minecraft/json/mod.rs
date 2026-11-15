#![allow(dead_code, unused_imports, non_snake_case, non_camel_case_types)]
use serde::Deserialize;

macro_rules! impl_int_deserializers {
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



pub(crate) mod atlas;
pub(crate) mod block_state_definition;
pub(crate) mod equipment;
pub(crate) mod font;
pub(crate) mod item_definition;
pub(crate) mod lang;
pub(crate) mod model;
pub(crate) mod sounds;
