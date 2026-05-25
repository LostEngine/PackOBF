use crate::resource_pack::mapping::{IdCategory, GLOBAL_MAPPING, GLOBAL_ID_USAGE_COUNTER};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }

    pub fn parse(identifier: &str) -> Self {
        match identifier.split_once(':') {
            Some((namespace, path)) => Self::new(namespace, path),
            None => Self::new("minecraft", identifier),
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.namespace == "minecraft" {
            write!(f, "{}", self.path)
        } else {
            write!(f, "{}:{}", self.namespace, self.path)
        }
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::parse(&String::deserialize(deserializer)?))
    }
}

#[derive(Clone, Debug)]
pub struct TextureId(pub Identifier);

#[derive(Clone, Debug)]
pub struct TextureIdWithExt(pub Identifier);

#[derive(Clone, Debug)]
pub struct ModelId(pub Identifier);

#[derive(Clone, Debug)]
pub struct SoundId(pub Identifier);

macro_rules! impl_id_wrapper {
    ($name:ident, $category:expr $(, $suffix:expr)?) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mapping = GLOBAL_MAPPING.get().expect("Mappings not initialized");
                let id_str = self.0.to_string();
                let mapped = mapping.apply_mapping(&id_str, $category);

                write!(f, "{}{}", mapped, concat!($($suffix)?))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let string = String::deserialize(deserializer)?;
                let path = {
                    #[allow(unused_mut)]
                    let mut s = string;
                    $(
                        if s.ends_with($suffix) {
                            s.truncate(s.len() - $suffix.len());
                        }
                    )?
                    s
                };
                let id = Identifier::parse(&path);
                let counter = GLOBAL_ID_USAGE_COUNTER.get().expect("Counter not initialized");
                counter.increment_counter(id.to_string(), $category);

                Ok(Self(id))
            }
        }
    };
}

impl_id_wrapper!(TextureId, IdCategory::Texture);
impl_id_wrapper!(TextureIdWithExt, IdCategory::Texture, ".png");
impl_id_wrapper!(ModelId, IdCategory::Model);
impl_id_wrapper!(SoundId, IdCategory::Sound);
