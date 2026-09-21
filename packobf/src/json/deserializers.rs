use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize_optional_i32<'a, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'a>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrFloat {
        Int(i32),
        Float(f64),
    }

    match Option::<IntOrFloat>::deserialize(deserializer)? {
        Some(IntOrFloat::Int(i)) => Ok(Some(i)),
        Some(IntOrFloat::Float(f)) => Ok(Some(f as i32)),
        None => Ok(None),
    }
}

pub(crate) fn deserialize_i32<'a, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'a>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrFloat {
        Int(i32),
        Float(f64),
    }

    match IntOrFloat::deserialize(deserializer)? {
        IntOrFloat::Int(i) => Ok(i),
        IntOrFloat::Float(f) => Ok(f as i32),
    }
}

pub(crate) fn deserialize_u32<'a, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'a>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrFloat {
        Int(i32),
        UnsignedInt(u32),
        Float(f64),
    }

    match IntOrFloat::deserialize(deserializer)? {
        IntOrFloat::UnsignedInt(u) => Ok(u),
        IntOrFloat::Int(i) => Ok(i as u32),
        IntOrFloat::Float(f) => Ok(f as u32),
    }
}
