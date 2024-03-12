use std::{
    array::TryFromSliceError,
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter}
};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b
};
use datasize::DataSize;
use schemars::JsonSchema;
use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};

use odra_core::casper_types::bytesrepr::FromBytes;
use odra_core::casper_types::{
    bytesrepr::{self, ToBytes},
    checksummed_hex, CLType, CLTyped
};
use hex_fmt::HexFmt;
use crate::casper_node_port::error::HashingError;

/// The output of the hash function.
#[derive(Copy, Clone, DataSize, Ord, PartialOrd, Eq, PartialEq, Hash, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(with = "String", description = "Hex-encoded hash digest.")]
pub struct Digest(#[schemars(skip, with = "String")] [u8; Digest::LENGTH]);

impl Digest {
    /// The number of bytes in a `Digest`.
    pub const LENGTH: usize = 32;

    /// Sentinel hash to be used for hashing options in the case of `None`.
    pub const SENTINEL_NONE: Digest = Digest([0u8; Digest::LENGTH]);
    /// Sentinel hash to be used by `hash_slice_rfold`. Terminates the fold.
    pub const SENTINEL_RFOLD: Digest = Digest([1u8; Digest::LENGTH]);
    /// Sentinel hash to be used by `hash_merkle_tree` in the case of an empty list.
    pub const SENTINEL_MERKLE_TREE: Digest = Digest([2u8; Digest::LENGTH]);

    /// Creates a 32-byte BLAKE2b hash digest from a given a piece of data.
    pub fn hash<T: AsRef<[u8]>>(data: T) -> Digest {
        Self::blake2b_hash(data)
    }

    /// Creates a 32-byte BLAKE2b hash digest from a given a piece of data
    pub(crate) fn blake2b_hash<T: AsRef<[u8]>>(data: T) -> Digest {
        let mut ret = [0u8; Digest::LENGTH];
        // NOTE: Safe to unwrap here because our digest length is constant and valid
        let mut hasher = VarBlake2b::new(Digest::LENGTH).unwrap();
        hasher.update(data);
        hasher.finalize_variable(|hash| ret.clone_from_slice(hash));
        Digest(ret)
    }

    /// Returns the underlying BLAKE2b hash bytes
    pub fn value(&self) -> [u8; Digest::LENGTH] {
        self.0
    }

    /// Converts the underlying BLAKE2b hash digest array to a `Vec`
    pub fn into_vec(self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Returns a `Digest` parsed from a hex-encoded `Digest`.
    pub fn from_hex<T: AsRef<[u8]>>(hex_input: T) -> Result<Self, HashingError> {
        let bytes = checksummed_hex::decode(&hex_input).map_err(HashingError::Base16DecodeError)?;
        let slice: [u8; Self::LENGTH] = bytes
            .try_into()
            .map_err(|_| HashingError::IncorrectDigestLength(hex_input.as_ref().len()))?;
        Ok(Digest(slice))
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:10}", HexFmt(&self.0))
    }
}

impl Debug for Digest {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", base16::encode_lower(&self.0))
    }
}

impl CLTyped for Digest {
    fn cl_type() -> CLType {
        CLType::ByteArray(Digest::LENGTH as u32)
    }
}

impl From<[u8; Digest::LENGTH]> for Digest {
    fn from(arr: [u8; Digest::LENGTH]) -> Self {
        Digest(arr)
    }
}

impl<'a> TryFrom<&'a [u8]> for Digest {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Digest, Self::Error> {
        <[u8; Digest::LENGTH]>::try_from(slice).map(Digest)
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Digest> for [u8; Digest::LENGTH] {
    fn from(hash: Digest) -> Self {
        hash.0
    }
}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            base16::encode_lower(&self.0).serialize(serializer)
        } else {
            // This is to keep backwards compatibility with how HexForm encodes
            // byte arrays. HexForm treats this like a slice.
            self.0[..].serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let hex_string = String::deserialize(deserializer)?;
            let bytes =
                checksummed_hex::decode(hex_string.as_bytes()).map_err(SerdeError::custom)?;
            let data =
                <[u8; Digest::LENGTH]>::try_from(bytes.as_ref()).map_err(SerdeError::custom)?;
            Ok(Digest::from(data))
        } else {
            let data = <Vec<u8>>::deserialize(deserializer)?;
            Digest::try_from(data.as_slice()).map_err(D::Error::custom)
        }
    }
}

impl ToBytes for Digest {
    #[inline(always)]
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    #[inline(always)]
    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }

    #[inline(always)]
    fn write_bytes(&self, writer: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        writer.extend_from_slice(&self.0);
        Ok(())
    }
}

impl FromBytes for Digest {
    #[inline(always)]
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        FromBytes::from_bytes(bytes).map(|(arr, rem)| (Digest(arr), rem))
    }
}
