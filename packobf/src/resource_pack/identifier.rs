use crate::resource_pack::mapping::{IdCategory, GLOBAL_MAPPING};
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

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapping = GLOBAL_MAPPING.get().expect("Mappings not initialized");
        let id = if self.0.namespace == "minecraft" {
            format!("{}", self.0.path)
        } else {
            format!("{}:{}", self.0.namespace, self.0.path)
        };
        write!(f, "{}", mapping.apply_mapping(&id, IdCategory::Texture))
    }
}

impl Serialize for TextureId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TextureId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            0: Identifier::parse(&String::deserialize(deserializer)?),
        })
    }
}

#[derive(Clone, Debug)]
pub struct TextureIdWithExt(pub Identifier);

impl std::fmt::Display for TextureIdWithExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapping = GLOBAL_MAPPING.get().expect("Mappings not initialized");
        let id = if self.0.namespace == "minecraft" {
            format!("{}", self.0.path)
        } else {
            format!("{}:{}", self.0.namespace, self.0.path)
        };
        write!(f, "{}.png", mapping.apply_mapping(&id, IdCategory::Texture))
    }
}

impl Serialize for TextureIdWithExt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TextureIdWithExt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            0: Identifier::parse(&String::deserialize(deserializer)?.replace(".png", "")),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ModelId(pub Identifier);

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapping = GLOBAL_MAPPING.get().expect("Mappings not initialized");
        let id = if self.0.namespace == "minecraft" {
            format!("{}", self.0.path)
        } else {
            format!("{}:{}", self.0.namespace, self.0.path)
        };
        write!(f, "{}", mapping.apply_mapping(&id, IdCategory::Model))
    }
}

impl Serialize for ModelId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ModelId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            0: Identifier::parse(&String::deserialize(deserializer)?),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SoundId(pub Identifier);

impl std::fmt::Display for SoundId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mapping = GLOBAL_MAPPING.get().expect("Mappings not initialized");
        let id = if self.0.namespace == "minecraft" {
            format!("{}", self.0.path)
        } else {
            format!("{}:{}", self.0.namespace, self.0.path)
        };
        write!(f, "{}", mapping.apply_mapping(&id, IdCategory::Sound))
    }
}

impl Serialize for SoundId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SoundId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            0: Identifier::parse(&String::deserialize(deserializer)?),
        })
    }
}
