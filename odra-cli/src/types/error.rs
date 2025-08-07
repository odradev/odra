use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid hex string")]
    InvalidHexString,
    #[error("Invalid binary string")]
    InvalidBinaryString,
    #[error("Hex decode error")]
    HexDecode,
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    BigUint(String),
    #[error("Serialization error")]
    Serialization,
    #[error("Deserialization error")]
    Deserialization,
    #[error("Invalid URef")]
    InvalidURef,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid map")]
    InvalidMap,
    #[error("Formatting error:\nexpected formats\n{0}")]
    Formatting(Format),
    #[error("Invalid event member type {0}")]
    InvalidEventMemberType(String),
    #[error("Invalid event type {0}")]
    InvalidEventType(String),
    #[error("Unexpected type while decoding")]
    UnexpectedType,
    #[error("Unexpected error: {0}")]
    Other(String)
}

#[derive(PartialEq)]
pub enum Format {
    Result,
    Option,
    Tuple { actual: usize, expected: usize },
    Map,
    ByteArray,
    InvalidLength { actual: usize, expected: usize },
    PatternLength { actual: usize, expected: usize },
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
            Format::Map => vec![String::from("'key1:value1,key2:value2,...'")],
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
