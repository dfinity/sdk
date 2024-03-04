use candid::Deserialize;
use schemars::JsonSchema;
use serde::Serialize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum SerdeVec<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> SerdeVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(t) => vec![t],
            Self::Many(ts) => ts,
        }
    }
}

impl<T> Default for SerdeVec<T> {
    fn default() -> Self {
        Self::Many(vec![])
    }
}

#[derive(Serialize, serde::Deserialize)]
#[serde(untagged)]
enum PossiblyStrInner<T> {
    NotStr(T),
    Str(String),
}

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, JsonSchema)]
#[serde(try_from = "PossiblyStrInner<T>")]
pub struct PossiblyStr<T>(pub T)
where
    T: FromStr,
    T::Err: Display;

impl<T> TryFrom<PossiblyStrInner<T>> for PossiblyStr<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Error = T::Err;
    fn try_from(inner: PossiblyStrInner<T>) -> Result<Self, Self::Error> {
        match inner {
            PossiblyStrInner::NotStr(t) => Ok(Self(t)),
            PossiblyStrInner::Str(str) => T::from_str(&str).map(Self),
        }
    }
}
