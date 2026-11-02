use std::fmt::{Debug, Display};
use thiserror::Error;

/// Errors of converting a value between its text format and bytes.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    /// A hex value is malformed or has no `0x` prefix.
    #[error("Invalid hex string")]
    InvalidHexString,
    /// A binary value is malformed.
    #[error("Invalid binary string")]
    InvalidBinaryString,
    /// A hex value cannot be decoded.
    #[error("Hex decode error")]
    HexDecode,
    /// A value cannot be parsed as its type.
    #[error("{0}")]
    Parse(String),
    /// A big integer (`U128`, `U256`, `U512`) cannot be parsed.
    #[error("{0}")]
    BigUint(String),
    /// A value cannot be serialized.
    #[error("Serialization error")]
    Serialization,
    /// Bytes cannot be deserialized as the expected type.
    #[error("Deserialization error")]
    Deserialization,
    /// A `URef` is malformed.
    #[error("Invalid URef")]
    InvalidURef,
    /// A public key is malformed.
    #[error("Invalid public key")]
    InvalidPublicKey,
    /// A map is malformed.
    #[error("Invalid map")]
    InvalidMap,
    /// A value does not match the expected text format.
    #[error("Formatting error:\nexpected formats\n{0}")]
    Formatting(Format),
    /// An event member has an unsupported type.
    #[error("Invalid event member type {0}")]
    InvalidEventMemberType(String),
    /// An event is not defined in the custom types.
    #[error("Invalid event type {0}")]
    InvalidEventType(String),
    /// The type cannot be decoded.
    #[error("Unexpected type while decoding")]
    UnexpectedType,
    /// Any other error.
    #[error("Unexpected error: {0}")]
    Other(String)
}

/// The expected text format of a value, see [Error::Formatting].
#[derive(PartialEq)]
pub enum Format {
    /// `ok:{value}` or `err:{value}`.
    Result,
    /// `none` or `some:{value}`.
    Option,
    /// A tuple with the wrong number of elements.
    Tuple {
        /// The number of elements given.
        actual: usize,
        /// The number of elements of the tuple.
        expected: usize
    },
    /// `key1=value1,key2=value2,...`.
    Map,
    /// `0x000102...`, `0x00,0x01,...` or `0,1,...`.
    ByteArray,
    /// A byte array of the wrong length.
    InvalidLength {
        /// The length given.
        actual: usize,
        /// The length of the array.
        expected: usize
    },
    /// A byte pattern that does not divide the length of the array.
    PatternLength {
        /// The length of the pattern.
        actual: usize,
        /// The length of the array.
        expected: usize
    },
    /// `0x00`, `0b00000001` or a decimal `0-255`.
    U8
}

impl Debug for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.as_string_vec().join("\n");
        f.write_str(&msg)
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.as_string_vec().join("\n");
        f.write_str(&msg)
    }
}

impl Format {
    fn as_string_vec(&self) -> Vec<String> {
        match self {
            Format::Result => vec![String::from("'ok:{value}'"), String::from("'err:{value}'")],
            Format::Option => vec![String::from("'none'"), String::from("'some:{value}'")],
            Format::Tuple { actual, expected } => vec![format!(
                "expected tuple with {} elements, found {}",
                expected, actual
            )],
            Format::Map => vec![String::from("'key1=value1,key2=value2,...'")],
            Format::ByteArray => vec![
                String::from("'0x000102...'"),
                String::from("'0x00,0x01,...'"),
                String::from("'0,1,...'"),
            ],
            Format::InvalidLength { actual, expected } => {
                vec![format!("expected length {}, found {}", expected, actual)]
            }
            Format::U8 => vec![
                String::from("'0x00'"),
                String::from("'0b00000001'"),
                String::from("'1'"),
                String::from("'255'"),
            ],
            Format::PatternLength { actual, expected } => vec![format!(
                "pattern length {} does not divide expected length {}",
                actual, expected
            )]
        }
    }
}

/// Errors of [super::decode] and [super::decode_event].
#[derive(Debug, Error)]
pub enum DecodeError {
    /// A value cannot be converted.
    #[error(transparent)]
    Value(#[from] Error),
    /// The bytes do not match the type.
    #[error("Decoding error: {0}")]
    Decoding(String),
    /// A custom type is not in the given [super::CustomTypeSet].
    #[error("Unknown type `{0}`")]
    UnknownType(String)
}
