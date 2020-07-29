use crate::types::blob::Blob;
// use crc8::Crc8;
// use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str};
use std::fmt::Write as FmtWrite;

/// Prefix for [textual form of ID](https://docs.dfinity.systems/spec/public/#textual-ids)
// const CANISTER_ID_TEXT_PREFIX: &str = "ic:";

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
    NotBase32,
    Wrong,
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
    pub fn from_text<S: std::string::ToString + AsRef<[u8]>>(text: S) -> Result<CanisterId, TextualCanisterIdError> {
        if text.as_ref().len() < 4 {
            Err(TextualCanisterIdError::TooShort)
        } else {
            // Strategy: Parse very liberally, then pretty-print and compare output
            // This is both simpler and yields better error messages

            let mut s = text.to_string();
            s.make_ascii_lowercase();
            s.retain(|c| c.is_ascii_alphanumeric());
            match base32::decode(base32::Alphabet::RFC4648 { padding: false }, &s) {
                Some(mut bytes) => {
                    if bytes.len() < 4 {
                        return Err(TextualCanisterIdError::TooShort);
                    }
                    let result = CanisterId::from_bytes(bytes.split_off(4));
                    let result_str = result.to_text();
                    // let expected = format!("{}", result);

                    // if text.to_string() != result_str {
                    //     // return Err(TextualCanisterIdError::Wrong { expected });
                    //     return Err(TextualCanisterIdError::Wrong);
                    // }
                    Ok(result)
                }
                None => Err(TextualCanisterIdError::NotBase32),
            }


        }
    }

    pub fn to_text(&self) -> String {
        let blob = &self.as_bytes();
        // calc checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(blob);
        let checksum = hasher.finalize();

        // combine blobs
        let mut bytes = vec![];
        bytes.extend(&(checksum.to_be_bytes().to_vec()));
        bytes.extend_from_slice(blob);

        // base32
        let mut s = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes);
        s.make_ascii_lowercase();

        // write out string with dashes
        while s.len() > 5 {
            // to bad split_off does not work the other way
            let rest = s.split_off(5);
            let mut f = String::new();
            write!(f, "{}-", s);
            s = rest;
        }
        format!("{}", s)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
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
    fn parse_management_canister_ok() {
        assert_eq!(
            CanisterId::from_text("aaaaa-aa").unwrap(),
            CanisterId(Blob::empty())
        );
    }

    // #[test]
    // fn check_serialize_deserialize() {
    //     let id = CanisterId::from_text("ic:ABCD01A7").unwrap();

    //     // Use cbor serialization.
    //     let vec = serde_cbor::to_vec(&id).unwrap();
    //     let value = serde_cbor::from_slice(vec.as_slice()).unwrap();

    //     assert_eq!(id, value);
    // }

    // #[test]
    // fn text_form_matches_public_spec() {
    //     // See example here: https://docs.dfinity.systems/spec/public/#textual-ids
    //     let textid = "ic:ABCD01A7";
    //     match CanisterId::from_text(textid) {
    //         Ok(ref cid) => assert_eq!(CanisterId::to_text(cid), textid),
    //         Err(_) => unreachable!(),
    //     }
    // }

    // #[test]
    // fn text_form() {
    //     let cid: CanisterId = CanisterId::from(Blob::from(vec![1, 8, 64, 255].as_slice()));
    //     let text = cid.to_text();
    //     let cid2 = CanisterId::from_text(&text).unwrap();
    //     assert_eq!(cid, cid2);
    //     assert_eq!(text, "ic:010840FFEF");
    // }
}
