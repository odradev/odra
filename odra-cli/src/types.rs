use std::{
    fmt::{Debug, Display},
    str::FromStr
};

use odra::schema::casper_contract_schema::NamedCLType;
use odra::{
    casper_types::{
        bytesrepr::{
            FromBytes, ToBytes, OPTION_NONE_TAG, OPTION_SOME_TAG, RESULT_ERR_TAG, RESULT_OK_TAG
        },
        AsymmetricType, CLType, Key, PublicKey, URef, U128, U256, U512
    },
    Address
};

use thiserror::Error;

const PREFIX_ERROR: &str = "err:";
const PREFIX_OK: &str = "ok:";
// const PREFIX_NONE: &str = "none";
// const PREFIX_SOME: &str = "some:";

pub enum Format {
    Result,
    Option,
    Tuple { actual: usize, expected: usize },
    Map,
    ByteArray
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
            ]
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid hex string")]
    InvalidHexString,
    #[error("Hex decode error")]
    HexDecode,
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("BigUint error: {0}")]
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
    #[error("Unexpected error: {0}")]
    Other(String)
}

type TypeResult<T> = Result<T, Error>;

macro_rules! call_from_bytes {
    ($ty:ty, $value:ident) => {
        <$ty as FromBytes>::from_bytes($value)
            .map(|(v, rem)| (v.to_string(), rem))
            .map_err(|_| Error::Serialization)
    };
}

macro_rules! call_to_bytes {
    ($ty:ty, $value:ident) => {
        parse_value::<$ty>($value)?
            .to_bytes()
            .map_err(|_| Error::Serialization)
    };
}

macro_rules! big_int_to_bytes {
    ($ty:ident, $value:ident) => {
        $ty::from_dec_str($value)
            .map_err(|_| Error::BigUint($value.to_string()))?
            .to_bytes()
            .map_err(|_| Error::Serialization)
    };
}

pub(crate) fn parse_value<T: FromStr>(value: &str) -> TypeResult<T>
where
    <T as FromStr>::Err: Debug
{
    <T as FromStr>::from_str(value).map_err(|_| Error::Parse(value.to_string()))
}

pub(crate) fn named_cl_type_to_cl_type(ty: &NamedCLType) -> CLType {
    match ty {
        NamedCLType::Bool => CLType::Bool,
        NamedCLType::I32 => CLType::I32,
        NamedCLType::I64 => CLType::I64,
        NamedCLType::U8 => CLType::U8,
        NamedCLType::U32 => CLType::U32,
        NamedCLType::U64 => CLType::U64,
        NamedCLType::U128 => CLType::U128,
        NamedCLType::U256 => CLType::U256,
        NamedCLType::U512 => CLType::U512,
        NamedCLType::String => CLType::String,
        NamedCLType::Key => CLType::Key,
        NamedCLType::URef => CLType::URef,
        NamedCLType::PublicKey => CLType::PublicKey,
        NamedCLType::Option(ty) => CLType::Option(Box::new(named_cl_type_to_cl_type(ty))),
        NamedCLType::List(ty) => CLType::List(Box::new(named_cl_type_to_cl_type(ty))),
        NamedCLType::ByteArray(n) => CLType::ByteArray(*n),
        NamedCLType::Result { ok, err } => CLType::Result {
            ok: Box::new(named_cl_type_to_cl_type(ok)),
            err: Box::new(named_cl_type_to_cl_type(err))
        },
        NamedCLType::Map { key, value } => CLType::Map {
            key: Box::new(named_cl_type_to_cl_type(key)),
            value: Box::new(named_cl_type_to_cl_type(value))
        },
        NamedCLType::Tuple1(ty) => CLType::Tuple1([Box::new(named_cl_type_to_cl_type(&ty[0]))]),
        NamedCLType::Tuple2(ty) => CLType::Tuple2([
            Box::new(named_cl_type_to_cl_type(&ty[0])),
            Box::new(named_cl_type_to_cl_type(&ty[1]))
        ]),
        NamedCLType::Tuple3(ty) => CLType::Tuple3([
            Box::new(named_cl_type_to_cl_type(&ty[0])),
            Box::new(named_cl_type_to_cl_type(&ty[1])),
            Box::new(named_cl_type_to_cl_type(&ty[2]))
        ]),
        NamedCLType::Custom(_) => CLType::Any,
        NamedCLType::Unit => CLType::Unit
    }
}

pub(crate) fn vec_into_bytes(ty: &NamedCLType, input: Vec<&str>) -> TypeResult<Vec<u8>> {
    let mut result = to_bytes_or_err(input.len() as u32)?;
    for value in input {
        result.extend(into_bytes(ty, value)?);
    }
    Ok(result)
}

pub(crate) fn into_bytes(ty: &NamedCLType, input: &str) -> TypeResult<Vec<u8>> {
    match ty {
        NamedCLType::Bool => call_to_bytes!(bool, input),
        NamedCLType::I32 => call_to_bytes!(i32, input),
        NamedCLType::I64 => call_to_bytes!(i64, input),
        NamedCLType::U8 => call_to_bytes!(u8, input),
        NamedCLType::U32 => call_to_bytes!(u32, input),
        NamedCLType::U64 => call_to_bytes!(u64, input),
        NamedCLType::U128 => big_int_to_bytes!(U128, input),
        NamedCLType::U256 => big_int_to_bytes!(U256, input),
        NamedCLType::U512 => big_int_to_bytes!(U512, input),
        NamedCLType::String => call_to_bytes!(String, input),
        NamedCLType::Key => call_to_bytes!(Address, input),
        NamedCLType::URef => URef::from_formatted_str(input)
            .map_err(|_| Error::InvalidURef)?
            .to_bytes()
            .map_err(|_| Error::Serialization),
        NamedCLType::PublicKey => PublicKey::from_hex(input)
            .map_err(|_| Error::InvalidPublicKey)?
            .to_bytes()
            .map_err(|_| Error::Serialization),
        NamedCLType::Option(ty) => {
            if input == "none" {
                Ok(vec![OPTION_NONE_TAG])
            } else if input.starts_with("some:") {
                let value = input.strip_prefix("some:").unwrap();
                let mut result = vec![OPTION_SOME_TAG];
                result.extend(into_bytes(ty, value)?);
                Ok(result)
            } else {
                return Err(Error::Formatting(Format::Option));
            }
        }
        NamedCLType::Result { ok, err } => {
            let mut result = vec![];
            if input.starts_with(PREFIX_ERROR) {
                let value = input.strip_prefix(PREFIX_ERROR).unwrap();
                result.push(RESULT_ERR_TAG);
                result.extend(into_bytes(err, value)?);
                Ok(result)
            } else if input.starts_with(PREFIX_OK) {
                let value = input.strip_prefix(PREFIX_OK).unwrap();
                result.push(RESULT_OK_TAG);
                result.extend(into_bytes(ok, value)?);
                Ok(result)
            } else {
                Err(Error::Formatting(Format::Result))
            }
        }
        NamedCLType::Tuple1(ty) => into_bytes(&ty[0], input),
        NamedCLType::Tuple2(ty) => {
            let parts = input.split(',').collect::<Vec<_>>();
            if parts.len() != 2 {
                return Err(Error::Formatting(Format::Tuple {
                    actual: parts.len(),
                    expected: 2
                }));
            }
            let mut result = vec![];
            result.extend(into_bytes(&ty[0], parts[0])?);
            result.extend(into_bytes(&ty[1], parts[1])?);
            Ok(result)
        }
        NamedCLType::Tuple3(ty) => {
            let parts = input.split(',').collect::<Vec<_>>();
            if parts.len() != 3 {
                return Err(Error::Formatting(Format::Tuple {
                    actual: parts.len(),
                    expected: 3
                }));
            }
            let mut result = vec![];
            result.extend(into_bytes(&ty[0], parts[0])?);
            result.extend(into_bytes(&ty[1], parts[1])?);
            result.extend(into_bytes(&ty[2], parts[2])?);
            Ok(result)
        }
        NamedCLType::Unit => Ok(vec![]),
        NamedCLType::Map { key, value } => {
            let parts = input
                .split(',')
                .map(|part| {
                    let key_value = part.split(':').collect::<Vec<_>>();
                    if key_value.len() != 2 {
                        return Err(Error::Formatting(Format::Map));
                    }
                    Ok((key_value[0], key_value[1]))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let mut result = to_bytes_or_err(parts.len() as u32)?;
            for (k, v) in parts.iter() {
                result.extend(into_bytes(key, k)?);
                result.extend(into_bytes(value, v)?);
            }
            Ok(result)
        }
        NamedCLType::List(ty) => {
            let parts = input
                .split(',')
                .map(|part| into_bytes(ty, part))
                .collect::<Result<Vec<_>, _>>()?;
            let mut result = to_bytes_or_err(parts.len() as u32)?;
            for part in parts {
                result.extend(part);
            }
            Ok(result)
        }
        NamedCLType::ByteArray(n) => {
            let n = *n as usize;
            match parse_hex(input) {
                Ok(data) => {
                    validate_byte_array_size(n, data.len())?;
                    Ok(data)
                }
                Err(Error::InvalidHexString) => {
                    let parts = input.split(',').collect::<Vec<_>>();
                    validate_byte_array_size(n, parts.len())?;

                    if parts.iter().all(|s| s.starts_with("0x")) {
                        let bytes = parts
                            .iter()
                            .map(|part| parse_hex(input))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(bytes.concat())
                    } else {
                        parts
                            .iter()
                            .map(|part| part.parse::<u8>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|_| Error::Formatting(Format::ByteArray))
                    }
                }
                Err(e) => Err(e)
            }
        }
        NamedCLType::Custom(_) => unreachable!("should not be here")
    }
}

pub(crate) fn from_bytes<'a>(ty: &NamedCLType, input: &'a [u8]) -> TypeResult<(String, &'a [u8])> {
    match ty {
        NamedCLType::Bool => call_from_bytes!(bool, input),
        NamedCLType::I32 => call_from_bytes!(i32, input),
        NamedCLType::I64 => call_from_bytes!(i64, input),
        NamedCLType::U8 => call_from_bytes!(u8, input),
        NamedCLType::U32 => call_from_bytes!(u32, input),
        NamedCLType::U64 => call_from_bytes!(u64, input),
        NamedCLType::U128 => call_from_bytes!(U128, input),
        NamedCLType::U256 => call_from_bytes!(U256, input),
        NamedCLType::U512 => call_from_bytes!(U512, input),
        NamedCLType::String => call_from_bytes!(String, input),
        NamedCLType::Key => call_from_bytes!(Key, input),
        NamedCLType::URef => call_from_bytes!(URef, input),
        NamedCLType::PublicKey => call_from_bytes!(PublicKey, input),
        NamedCLType::Option(ty) => {
            if input.first() == Some(&OPTION_NONE_TAG) {
                Ok(("null".to_string(), input))
            } else {
                from_bytes(ty, &input[1..])
            }
        }
        NamedCLType::Result { ok, err } => {
            let (variant, rem) = from_bytes_or_err::<u8>(input)?;
            match variant {
                RESULT_ERR_TAG => {
                    let (value, rem) = from_bytes(err, rem)?;
                    Ok((format!("Err({})", value), rem))
                }
                RESULT_OK_TAG => {
                    let (value, rem) = from_bytes(ok, rem)?;
                    Ok((format!("Ok({})", value), rem))
                }
                _ => Err(Error::Other("Invalid result variant".to_string()))
            }
        }
        NamedCLType::Tuple1(ty) => {
            let v = from_bytes(&ty[0], input)?;
            Ok((format!("({},)", v.0), v.1))
        }
        NamedCLType::Tuple2(ty) => {
            let (v1, rem) = from_bytes(&ty[0], input)?;
            let (v2, rem) = from_bytes(&ty[1], rem)?;
            Ok((format!("({}, {})", v1, v2), rem))
        }
        NamedCLType::Tuple3(ty) => {
            let (v1, rem) = from_bytes(&ty[0], input)?;
            let (v2, rem) = from_bytes(&ty[1], rem)?;
            let (v3, rem) = from_bytes(&ty[2], rem)?;
            Ok((format!("({}, {}, {})", v1, v2, v3), rem))
        }
        NamedCLType::Unit => <() as FromBytes>::from_bytes(input)
            .map(|(_, rem)| ("".to_string(), rem))
            .map_err(|_| Error::Deserialization),

        NamedCLType::List(ty) => {
            let (num_keys, mut stream) = from_bytes_or_err::<u32>(input)?;
            let mut result = "".to_string();
            for _ in 0..num_keys {
                let (v, rem) = from_bytes(ty, stream)?;
                result.push_str(&v);
                result.push(',');
                stream = rem;
            }
            if num_keys > 0 {
                result.pop();
            }
            Ok((result, stream))
        }
        NamedCLType::ByteArray(n) => {
            let size = *n as usize;

            let mut hex = "0x".to_string();
            let mut dec = "".to_string();
            for val in input.iter().take(size) {
                dec.push_str(&format!("{}, ", val));
                hex.push_str(&format!("{:02x}", val));
            }

            // remove trailing comma
            if size > 0 {
                dec.pop();
                dec.pop();
            }

            Ok((format!("{} ({})", hex, dec), &input[size..]))
        }
        NamedCLType::Map { key, value } => {
            let (num_keys, mut stream) = from_bytes_or_err::<u32>(input)?;
            let mut result = "".to_string();
            for _ in 0..num_keys {
                let (k, rem) = from_bytes(key, stream)?;
                let (v, rem) = from_bytes(value, rem)?;
                result.push_str(&format!("{}:{}, ", k, v));
                stream = rem;
            }
            // remove trailing comma
            if num_keys > 0 {
                result.pop();
                result.pop();
            }
            Ok((result, stream))
        }
        NamedCLType::Custom(_) => unreachable!("should not be here")
    }
}

fn parse_hex(input: &str) -> TypeResult<Vec<u8>> {
    match input.strip_prefix("0x") {
        Some(data) => hex::decode(data).map_err(|_| Error::HexDecode),
        None => Err(Error::InvalidHexString)
    }
}

#[inline]
pub(crate) fn from_bytes_or_err<T: FromBytes>(input: &[u8]) -> TypeResult<(T, &[u8])> {
    T::from_bytes(input).map_err(|_| Error::Deserialization)
}

#[inline]
pub(crate) fn to_bytes_or_err<T: ToBytes>(input: T) -> TypeResult<Vec<u8>> {
    input.to_bytes().map_err(|_| Error::Serialization)
}

fn validate_byte_array_size(expected: usize, actual: usize) -> TypeResult<()> {
    if actual != expected {
        return Err(Error::Formatting(Format::ByteArray));
    }
    Ok(())
}
