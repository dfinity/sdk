use crate::types::request_id;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[cfg(test)]
use rand::{thread_rng, RngCore};

/// A binary "blob", i.e. a byte array
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    #[cfg(test)]
    pub fn random(size: usize) -> Blob {
        let mut rng = thread_rng();
        let mut v: Vec<u8> = Vec::with_capacity(size);
        rng.fill_bytes(v.as_mut_slice());

        Blob(v)
    }
}

impl From<&[u8]> for Blob {
    fn from(a: &[u8]) -> Blob {
        Blob(a.to_vec())
    }
}

/// Serialize into a u64 for now.
impl Serialize for Blob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

/// Simple visitor for deserialization from bytes. We don't support other number types
/// as there's no need for it.
struct BlobVisitor;

impl<'de> de::Visitor<'de> for BlobVisitor {
    type Value = Blob;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a binary large object (bytes)")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Blob::from(value))
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<S>(deserializer: S) -> Result<Blob, S::Error>
    where
        S: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BlobVisitor)
    }
}

impl From<request_id::RequestId> for Blob {
    fn from(rid: request_id::RequestId) -> Blob {
        Blob(rid.to_vec())
    }
}
