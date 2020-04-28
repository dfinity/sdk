use crate::types::blob::Blob;
use crc8::Crc8;
use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str};

/// Prefix for [textual form of ID](https://docs.dfinity.systems/spec/public/#textual-ids)
const CANISTER_ID_TEXT_PREFIX: &str = "ic:";

/// A Canister ID.
///
/// This type is described as a Blob in the public spec, but used as an integer in most
/// code samples (including this library). For now, we newtype it to abstract its usage
/// from a number, and will change its internal type when time comes.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct CanisterId(Blob);

#[derive(Clone, Debug)]
pub enum TextualCanisterIdError {
    TooShort,
    BadPrefix,
    BadChecksum,
    FromHexError(hex::FromHexError),
}

impl std::convert::From<hex::FromHexError> for TextualCanisterIdError {
    fn from(e: hex::FromHexError) -> Self {
        TextualCanisterIdError::FromHexError(e)
    }
}

impl CanisterId {
    /// Allow to move canister Ids in blobs.
    pub fn into_blob(self) -> Blob {
        self.0
    }

    pub fn from_bytes<S: AsRef<[u8]>>(bytes: S) -> CanisterId {
        CanisterId(Blob::from(bytes.as_ref()))
    }

    /// Parse the text format for canister IDs (e.g., `ic:010840FFAD`).
    ///
    /// The text format follows this
    /// [section of our public spec doc](https://docs.dfinity.systems/spec/public/#textual-ids).
    pub fn from_text<S: AsRef<[u8]>>(text: S) -> Result<CanisterId, TextualCanisterIdError> {
        if text.as_ref().len() < 4 {
            Err(TextualCanisterIdError::TooShort)
        } else {
            let (text_prefix, text_rest) = text.as_ref().split_at(3);
            match std::str::from_utf8(text_prefix) {
                Ok(ref s) => {
                    if s != &CANISTER_ID_TEXT_PREFIX {
                        return Err(TextualCanisterIdError::BadPrefix);
                    }
                }
                Err(_) => return Err(TextualCanisterIdError::BadPrefix),
            };
            match hex::decode(text_rest)?.as_slice().split_last() {
                None => Err(TextualCanisterIdError::TooShort),
                Some((last_byte, buf_head)) => {
                    let mut crc8 = Crc8::create_msb(0x07);
                    let checksum_byte: u8 = crc8.calc(buf_head, buf_head.len() as i32, 0);
                    if *last_byte == checksum_byte {
                        Ok(CanisterId(Blob::from(buf_head)))
                    } else {
                        Err(TextualCanisterIdError::BadChecksum)
                    }
                }
            }
        }
    }

    pub fn to_text(&self) -> String {
        let mut crc8 = Crc8::create_msb(0x07);
        let checksum_byte: u8 = crc8.calc(&(self.0).0, (self.0).0.len() as i32, 0);
        let mut buf = (self.0).0.clone();
        buf.push(checksum_byte);
        format!("{}{}", CANISTER_ID_TEXT_PREFIX, hex::encode_upper(buf))
    }
}

/// Serialize into a blob.
impl Serialize for CanisterId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CanisterId {
    fn deserialize<S>(deserializer: S) -> Result<CanisterId, S::Error>
    where
        S: Deserializer<'de>,
    {
        // TODO(DFN-862): move this to blobs
        Ok(CanisterId::from(Blob::deserialize(deserializer)?))
    }
}

/// Conversion of different types that should be coerce-able to Canister Ids.
impl From<Blob> for CanisterId {
    fn from(b: Blob) -> CanisterId {
        // We don't need to make a copy as this assume ownership.
        CanisterId(b)
    }
}

impl str::FromStr for CanisterId {
    type Err = TextualCanisterIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CanisterId::from_text(s)
    }
}

impl fmt::Display for CanisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

impl Into<Blob> for CanisterId {
    fn into(self) -> Blob {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_serialize_deserialize() {
        let id = CanisterId::from_text("ic:ABCD01A7").unwrap();

        // Use cbor serialization.
        let vec = serde_cbor::to_vec(&id).unwrap();
        let value = serde_cbor::from_slice(vec.as_slice()).unwrap();

        assert_eq!(id, value);
    }

    #[test]
    fn text_form_matches_public_spec() {
        // See example here: https://docs.dfinity.systems/spec/public/#textual-ids
        let textid = "ic:ABCD01A7";
        match CanisterId::from_text(textid) {
            Ok(ref cid) => assert_eq!(CanisterId::to_text(cid), textid),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn text_form() {
        let cid: CanisterId = CanisterId::from(Blob::from(vec![1, 8, 64, 255].as_slice()));
        let text = cid.to_text();
        let cid2 = CanisterId::from_text(&text).unwrap();
        assert_eq!(cid, cid2);
        assert_eq!(text, "ic:010840FFEF");
    }
}
