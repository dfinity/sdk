//! This module deals with computing Request IDs based on the content of a
//! message.
//!
//! We compute the `RequestId` according to the public spec, which
//! specifies it as a "sha256" digest.
//!
//! A single method is exported, to_request_id, which returns a RequestId
//! (a 256 bits slice) or an error.
use crate::types::request_id_error::{RequestIdError, RequestIdFromStringError};
use openssl::sha::Sha256;
use serde::{ser, Serialize, Serializer};
use std::collections::BTreeMap;
use std::iter::Extend;
use std::str::FromStr;

/// Type alias for a sha256 result (ie. a u256).
type Sha256Hash = [u8; 32];

/// A Request ID.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct RequestId(Sha256Hash);

impl RequestId {
    pub fn new(from: &[u8; 32]) -> RequestId {
        RequestId(*from)
    }

    pub(crate) fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl FromStr for RequestId {
    type Err = RequestIdFromStringError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        let mut blob: [u8; 32] = [0; 32];
        let vec = hex::decode(from).map_err(RequestIdFromStringError::FromHexError)?;
        if vec.len() != 32 {
            return Err(RequestIdFromStringError::InvalidSize(vec.len()));
        }

        blob.copy_from_slice(vec.as_slice());
        Ok(RequestId::new(&blob))
    }
}

impl From<RequestId> for String {
    fn from(id: RequestId) -> String {
        hex::encode(id.0)
    }
}

/// We only allow to serialize a Request ID.
impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_vec())
    }
}

/// A Serde Serializer that collects fields and values in order to hash them later.
/// We serialize the type to this structure, then use the trait to hash its content.
/// It is a simple state machine that contains 3 states:
///   1. The root value, which is a structure. If a value other than a structure is
///      serialized, this errors. This is determined by whether `fields` is Some(_).
///   2. The structure is being processed, and the value of a field is being
///      serialized. The field_value_hash will be set to Some(_).
///   3. The finish() function has been called and the hasher cannot be reused. The
///      hash should have been gotten at this point.
///
/// Inconsistent state are when a field is being serialized and `fields` is None, or
/// when a value (not struct) is being serialized and field_value_hash is None.
///
/// This will always fail on types that are unknown to the Request format (e.g. i8).
/// An UnsupportedTypeXXX error will be returned.
///
/// The only types that are supported right now are:
///   . Strings and string slices.
///   . Blobs (the newtype exported from this crate).
///   . A structure as the base level. Its typename and fields are not validated.
///
/// Additionally, this will fail if there are unsupported data structure, for example
/// if a UnitVariant of another type than Blob is used, or a structure inside a
/// structure.
///
/// This does not validate whether a message is valid. This is very important as
/// the message format might change faster than the ID calculation.
struct RequestIdSerializer {
    // We use a BTreeMap here as there is no indication that keys might not be duplicated,
    // and we want to make sure they're overwritten in that case.
    fields: Option<BTreeMap<Sha256Hash, Sha256Hash>>,
    field_key_hash: Option<Sha256Hash>, // Only used in maps, not structs.
    field_value_hash: Option<Sha256>,
    hasher: Sha256,
}

impl RequestIdSerializer {
    pub fn new() -> RequestIdSerializer {
        Default::default()
    }

    /// Finish the hashing and returns the RequestId for the structure that was
    /// serialized.
    ///
    /// This can only be called once (it borrows self). Since this whole class is not public,
    /// it should not be a problem.
    pub fn finish(mut self) -> Result<RequestId, RequestIdError> {
        if self.fields.is_some() {
            self.fields = None;
            Ok(RequestId(self.hasher.finish()))
        } else {
            Err(RequestIdError::EmptySerializer)
        }
    }

    /// Hash a single value, returning its sha256_hash. If there is already a value
    /// being hashed it will return an InvalidState. This cannot happen currently
    /// as we don't allow embedded structures, but is left as a safeguard when
    /// making changes.
    fn hash_value<T>(&mut self, value: &T) -> Result<Sha256Hash, RequestIdError>
    where
        T: ?Sized + Serialize,
    {
        if self.field_value_hash.is_some() {
            return Err(RequestIdError::InvalidState);
        }

        self.field_value_hash = Some(Sha256::new());
        value.serialize(&mut *self)?;
        if let Some(r) = self.field_value_hash.take() {
            Ok(r.finish())
        } else {
            Err(RequestIdError::InvalidState)
        }
    }
}

impl Default for RequestIdSerializer {
    fn default() -> RequestIdSerializer {
        RequestIdSerializer {
            fields: None,
            field_key_hash: None,
            field_value_hash: None,
            hasher: Sha256::new(),
        }
    }
}

/// See https://serde.rs/data-format.html for more information on how to implement a
/// custom data format.
impl<'a> ser::Serializer for &'a mut RequestIdSerializer {
    /// The output type produced by this `Serializer` during successful
    /// serialization. Most serializers that produce text or binary output
    /// should set `Ok = ()` and serialize into an [`io::Write`] or buffer
    /// contained within the `Serializer` instance. Serializers that build
    /// in-memory data structures may be simplified by using `Ok` to propagate
    /// the data structure around.
    ///
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    type Ok = ();

    /// The error type when some error occurs during serialization.
    type Error = RequestIdError;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// Serialize a `bool` value.
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeBool)
    }

    /// Serialize an `i8` value.
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeI8)
    }

    /// Serialize an `i16` value.
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeI16)
    }

    /// Serialize an `i32` value.
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeI32)
    }

    /// Serialize an `i64` value.
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeI64)
    }

    /// Serialize a `u8` value.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serialize a `u16` value.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serialize a `u32` value.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    /// Serialize a `u64` value.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut buffer = [0; 32];
        let mut writable = &mut buffer[..];
        leb128::write::unsigned(&mut writable, v)
            .map_err(|e| RequestIdError::Custom(format!("{}", e)))?;
        self.serialize_bytes(&buffer)
    }

    /// Serialize an `f32` value.
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeF32)
    }

    /// Serialize an `f64` value.
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeF64)
    }

    /// Serialize a character.
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeChar)
    }

    /// Serialize a `&str`.
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    /// Serialize a chunk of raw byte data.
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match self.field_value_hash {
            None => Err(RequestIdError::InvalidState),
            Some(ref mut hash) => {
                (*hash).update(v);
                Ok(())
            }
        }
    }

    /// Serialize a [`None`] value.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // Compute the hash as if it was empty string or blob.
        match self.field_value_hash {
            None => Err(RequestIdError::InvalidState),
            Some(ref mut _hash) => Ok(()),
        }
    }

    /// Serialize a [`Some(T)`] value.
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        // Compute the hash as if it was the value itself.
        value.serialize(self)
    }

    /// Serialize a `()` value.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeUnit)
    }

    /// Serialize a unit struct like `struct Unit` or `PhantomData<T>`.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypePhantomData)
    }

    /// Serialize a unit variant like `E::A` in `enum E { A, B }`.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(RequestIdError::UnsupportedTypeUnitVariant)
    }

    /// Serialize a newtype struct like `struct Millimeters(u8)`.
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        match name {
            "Blob" => value.serialize(self), // value is of type Vec<u8>.
            v => Err(RequestIdError::UnsupportedTypeNewtypeStruct(v.to_owned())),
        }
    }

    /// Serialize a newtype variant like `E::N` in `enum E { N(u8) }`.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(RequestIdError::UnsupportedTypeNewTypeVariant)
    }

    /// Begin to serialize a variably sized sequence. This call must be
    /// followed by zero or more calls to `serialize_element`, then a call to
    /// `end`.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    /// Begin to serialize a statically sized sequence whose length will be
    /// known at deserialization time without looking at the serialized data.
    /// This call must be followed by zero or more calls to `serialize_element`,
    /// then a call to `end`.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(RequestIdError::UnsupportedTypeTuple)
    }

    /// Begin to serialize a tuple struct like `struct Rgb(u8, u8, u8)`. This
    /// call must be followed by zero or more calls to `serialize_field`, then a
    /// call to `end`.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(RequestIdError::UnsupportedTypeTupleStruct)
    }

    /// Begin to serialize a tuple variant like `E::T` in `enum E { T(u8, u8)
    /// }`. This call must be followed by zero or more calls to
    /// `serialize_field`, then a call to `end`.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(RequestIdError::UnsupportedTypeTupleVariant)
    }

    /// Begin to serialize a map. This call must be followed by zero or more
    /// calls to `serialize_key` and `serialize_value`, then a call to `end`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // This is the same as struct, but unnamed. We will use the current_field field
        // here though, as serialize key and value are separate functions.
        if self.fields.is_none() {
            self.fields = Some(BTreeMap::new());
            Ok(self)
        } else {
            Err(RequestIdError::UnsupportedStructInsideStruct)
        }
    }

    /// Begin to serialize a struct like `struct Rgb { r: u8, g: u8, b: u8 }`.
    /// This call must be followed by zero or more calls to `serialize_field`,
    /// then a call to `end`.
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if self.fields.is_none() {
            self.fields = Some(BTreeMap::new());
            Ok(self)
        } else {
            Err(RequestIdError::UnsupportedStructInsideStruct)
        }
    }

    /// Begin to serialize a struct variant like `E::S` in `enum E { S { r: u8,
    /// g: u8, b: u8 } }`. This call must be followed by zero or more calls to
    /// `serialize_field`, then a call to `end`.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(RequestIdError::UnsupportedTypeStructVariant)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut RequestIdSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = RequestIdError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(RequestIdError::Custom(
            "Unsupported field type: SerializeTuple element.".to_string(),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(RequestIdError::Custom(
            "Unsupported field type: SerializeTupleStruct field.".to_string(),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(RequestIdError::Custom(
            "Unsupported field type: SerializeTupleVariant field.".to_string(),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.field_key_hash.is_some() {
            Err(RequestIdError::InvalidState)
        } else {
            let key_hash = self.hash_value(key)?;
            self.field_key_hash = Some(key_hash);
            Ok(())
        }
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value_hash = self.hash_value(value)?;

        match self.field_key_hash.take() {
            None => Err(RequestIdError::InvalidState),
            Some(key_hash) => match self.fields {
                None => Err(RequestIdError::InvalidState),
                Some(ref mut f) => {
                    f.insert(key_hash, value_hash);
                    Ok(())
                }
            },
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.field_value_hash.is_some() {
            return Err(RequestIdError::InvalidState);
        }

        let key_hash = self.hash_value(key)?;
        let value_hash = self.hash_value(value)?;

        match self.fields {
            None => Err(RequestIdError::InvalidState),
            Some(ref mut f) => {
                f.insert(key_hash, value_hash);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(fields) = &self.fields {
            // Sort the fields.
            let mut keyvalues: Vec<Vec<u8>> = fields
                .keys()
                .zip(fields.values())
                .map(|(k, v)| {
                    let mut x = k.to_vec();
                    x.extend(v);
                    x
                })
                .collect();
            keyvalues.sort();

            for kv in keyvalues {
                self.hasher.update(&kv);
            }

            Ok(())
        } else {
            Err(RequestIdError::InvalidState)
        }
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut RequestIdSerializer {
    type Ok = ();
    type Error = RequestIdError;

    fn serialize_field<T>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(RequestIdError::Custom(
            "Unsupported field type: SerializeStructVariant field.".to_string(),
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Derive the request ID from a serializable data structure.
///
/// See https://hydra.dfinity.systems//build/268411/download/1/dfinity/spec/public/index.html#api-request-id
pub fn to_request_id<'a, V>(value: &V) -> Result<RequestId, RequestIdError>
where
    V: 'a + Serialize,
{
    let mut serializer = RequestIdSerializer::new();
    value.serialize(&mut serializer)?;
    serializer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Blob, CanisterId};

    /// The actual example used in the public spec in the Request ID section.
    #[test]
    fn public_spec_example() {
        #[derive(Serialize)]
        struct PublicSpecExampleStruct {
            request_type: &'static str,
            canister_id: CanisterId,
            method_name: &'static str,
            arg: Blob,
        };
        let data = PublicSpecExampleStruct {
            request_type: "call",
            canister_id: CanisterId::from_bytes(&[0, 0, 0, 0, 0, 0, 0x04, 0xD2]), // 1234 in u64
            method_name: "hello",
            arg: Blob(b"DIDL\x00\xFD*".to_vec()),
        };

        // Hash taken from the example on the public spec.
        let request_id = to_request_id(&data).unwrap();
        assert_eq!(
            hex::encode(request_id.0.to_vec()),
            "8781291c347db32a9d8c10eb62b710fce5a93be676474c42babc74c51858f94b"
        );
    }

    /// The same example as above, except we use the ApiClient enum newtypes.
    #[test]
    fn public_spec_example_api_client() {
        #[derive(Serialize)]
        #[serde(rename_all = "snake_case")]
        #[serde(tag = "request_type")]
        enum PublicSpec {
            Call {
                canister_id: CanisterId,
                method_name: String,
                arg: Option<Blob>,
            },
        }
        let data = PublicSpec::Call {
            canister_id: CanisterId::from_bytes(&[0, 0, 0, 0, 0, 0, 0x04, 0xD2]), // 1234 in u64
            method_name: "hello".to_owned(),
            arg: Some(Blob(b"DIDL\x00\xFD*".to_vec())),
        };

        // Hash taken from the example on the public spec.
        let request_id = to_request_id(&data).unwrap();
        assert_eq!(
            hex::encode(request_id.0.to_vec()),
            "8781291c347db32a9d8c10eb62b710fce5a93be676474c42babc74c51858f94b"
        );
    }
}
