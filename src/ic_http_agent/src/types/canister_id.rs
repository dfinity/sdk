use crate::types::blob::Blob;
use byteorder::{BigEndian, ByteOrder};
use crc8::Crc8;
use hex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, num, str};

/// A Canister ID.
///
/// This type is described as a Blob in the public spec, but used as an integer in most
/// code samples (including this library). For now, we newtype it to abstract its usage
/// from a number, and will change its internal type when time comes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanisterId(Blob);

impl CanisterId {
    pub(crate) fn from_u64(v: u64) -> CanisterId {
        let mut buf = [0 as u8; 8];
        BigEndian::write_u64(&mut buf, v);
        CanisterId(Blob(buf.to_vec()))
    }

    pub(crate) fn as_u64(&self) -> u64 {
        BigEndian::read_u64((self.0).0.as_slice())
    }

    /// Allow to move canister Ids in blobs.
    pub fn into_blob(self) -> Blob {
        self.0
    }

    // Text format for canister IDs follows this [section of our public spec doc](https://docs.dfinity.systems/spec/public/#textual-ids).

    // todo: this error code and this error type are both wrong;
    // IMO, we need our own, since we have our own spec (see url above).

    // todo: to follow the real text format, we need to introduce and eliminate the "ic:" prefix.  Currently, we assume it is absent in from_hex, and do not add it in to_hex.

    pub fn from_hex<S: AsRef<[u8]>>(h: S) -> Result<CanisterId, hex::FromHexError> {
        match hex::decode(h)?.as_slice().split_last() {
            None => Err(hex::FromHexError::InvalidStringLength),
            Some((last_byte, buf_head)) => {
                let mut crc8 = Crc8::create_msb(17);
                let checksum_byte: u8 = crc8.calc(buf_head, buf_head.len() as i32, 0);
                if *last_byte == checksum_byte {
                    Ok(CanisterId(Blob::from(buf_head)))
                } else {
                    Err(hex::FromHexError::InvalidStringLength)
                }
            }
        }
    }

    pub fn to_hex(&self) -> String {
        let mut crc8 = Crc8::create_msb(17);
        let checksum_byte: u8 = crc8.calc(&(self.0).0, (self.0).0.len() as i32, 0);
        let mut buf = (self.0).0.clone();
        buf.push(checksum_byte);
        hex::encode_upper(buf)
    }
}

/// Serialize into a blob.
impl Serialize for CanisterId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO(DFN-862): move this to blobs
        serializer.serialize_u64(self.as_u64())
    }
}

impl<'de> Deserialize<'de> for CanisterId {
    fn deserialize<S>(deserializer: S) -> Result<CanisterId, S::Error>
    where
        S: Deserializer<'de>,
    {
        // TODO(DFN-862): move this to blobs
        Ok(CanisterId::from_u64(u64::deserialize(deserializer)?))
    }
}

/// Conversion of different types that should be coerce-able to Canister Ids.
impl From<Blob> for CanisterId {
    fn from(b: Blob) -> CanisterId {
        // We don't need to make a copy as this assume ownership.
        CanisterId(b)
    }
}

impl From<u64> for CanisterId {
    fn from(n: u64) -> CanisterId {
        // We don't need to make a copy as this assume ownership.
        CanisterId::from_u64(n)
    }
}

impl str::FromStr for CanisterId {
    type Err = num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(CanisterId::from_u64(u64::from_str(s)?))
    }
}

impl fmt::Display for CanisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CanisterId({})", self.to_hex())
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
        let id = CanisterId::from_u64(88827);

        // Use cbor serialization.
        let vec = serde_cbor::to_vec(&id).unwrap();
        let value = serde_cbor::from_slice(vec.as_slice()).unwrap();

        assert_eq!(id, value);
    }

    #[test]
    fn hex_form() {
        let cid: CanisterId = CanisterId::from(Blob::from(vec![1, 8, 64, 255].as_slice()));
        let hex = cid.to_hex();
        let cid2 = CanisterId::from_hex(&hex).unwrap();
        assert_eq!(cid, cid2);
        assert_eq!(hex, "010840FFAD");
    }
}
