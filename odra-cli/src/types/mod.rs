use std::{fmt::Debug, str::FromStr};

use odra::schema::casper_contract_schema::NamedCLType;
use odra::{
    casper_types::{
        bytesrepr::{
            FromBytes, ToBytes, OPTION_NONE_TAG, OPTION_SOME_TAG, RESULT_ERR_TAG, RESULT_OK_TAG
        },
        AsymmetricType, CLType, PublicKey, URef, U128, U256, U512
    },
    prelude::Address
};

mod decoder;
mod error;

#[cfg(test)]
mod into_bytes;

pub(crate) use decoder::{decode, decode_event};
pub(crate) use error::{Error, Format};

const PREFIX_ERROR: &str = "err:";
const PREFIX_OK: &str = "ok:";
const PREFIX_HEX: &str = "0x";
const PREFIX_BINARY: &str = "0b";
const PREFIX_SOME: &str = "some:";
const PREFIX_NONE: &str = "none";

type TypeResult<T> = Result<T, Error>;

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
            .map_err(|e| Error::BigUint(e.to_string()))?
            .to_bytes()
            .map_err(|_| Error::Serialization)
    };
}

pub(crate) fn format_variant_list(variants: &[(String, u16)]) -> String {
    variants
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join("|")
}

pub(crate) fn format_type_hint(ty: &NamedCLType) -> String {
    match ty {
        NamedCLType::Bool => "true|false".into(),
        NamedCLType::I32 | NamedCLType::I64 => "INT".into(),
        NamedCLType::U8 => "0-255|0x00|0b00000000".into(),
        NamedCLType::U32 | NamedCLType::U64 => "UINT".into(),
        NamedCLType::U128 | NamedCLType::U256 | NamedCLType::U512 => "DECIMAL".into(),
        NamedCLType::String => "TEXT".into(),
        NamedCLType::Key => "hash-...|account-hash-...".into(),
        NamedCLType::URef => "uref-...-NNN".into(),
        NamedCLType::PublicKey => "HEX_PUBLIC_KEY".into(),
        NamedCLType::Option(t) => format!("none|some:{}", format_type_hint(t)),
        NamedCLType::Result { ok, err } => {
            format!("ok:{}|err:{}", format_type_hint(ok), format_type_hint(err))
        }
        NamedCLType::List(box NamedCLType::U8) => "BYTE,BYTE,...".into(),
        NamedCLType::List(t) => format!("{} (repeatable)", format_type_hint(t)),
        NamedCLType::Map { key, value } => {
            let k = format_type_hint(key);
            let v = format_type_hint(value);
            format!("{}={}[,{}={}]", k, v, k, v)
        }
        NamedCLType::Tuple1(t) => format_type_hint(&t[0]),
        NamedCLType::Tuple2(t) => {
            format!("{}:{}", format_type_hint(&t[0]), format_type_hint(&t[1]))
        }
        NamedCLType::Tuple3(t) => format!(
            "{}:{}:{}",
            format_type_hint(&t[0]),
            format_type_hint(&t[1]),
            format_type_hint(&t[2])
        ),
        NamedCLType::ByteArray(_) => "0xHEX".into(),
        NamedCLType::Unit => "(empty)".into(),
        NamedCLType::Custom(name) => name.clone()
    }
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

pub(crate) fn into_bytes(ty: &NamedCLType, input: &str) -> TypeResult<Vec<u8>> {
    match ty {
        NamedCLType::Bool => call_to_bytes!(bool, input),
        NamedCLType::I32 => call_to_bytes!(i32, input),
        NamedCLType::I64 => call_to_bytes!(i64, input),
        NamedCLType::U8 => {
            if let Some(hex) = input.strip_prefix(PREFIX_HEX) {
                u8::from_str_radix(hex, 16)
                    .map_err(|_| Error::InvalidHexString)
                    .map(|byte| vec![byte])
            } else if let Some(bits) = input.strip_prefix(PREFIX_BINARY) {
                let byte = u8::from_str_radix(bits, 2).map_err(|_| Error::Serialization)?;
                Ok(vec![byte])
            } else {
                // Fallback to parsing as decimal
                if let Ok(byte) = input.parse::<u8>() {
                    Ok(vec![byte])
                } else {
                    Err(Error::Formatting(Format::U8))
                }
            }
        }
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
            if input == PREFIX_NONE {
                Ok(vec![OPTION_NONE_TAG])
            } else if input.starts_with(PREFIX_SOME) {
                let value = strip_prefix_or_err(input, PREFIX_SOME)?;
                let mut result = vec![OPTION_SOME_TAG];
                result.extend(into_bytes(ty, value)?);
                Ok(result)
            } else {
                Err(Error::Formatting(Format::Option))
            }
        }
        NamedCLType::Result { ok, err } => {
            let mut result = vec![];
            if input.starts_with(PREFIX_ERROR) {
                let value = strip_prefix_or_err(input, PREFIX_ERROR)?;
                result.push(RESULT_ERR_TAG);
                result.extend(into_bytes(err, value)?);
                Ok(result)
            } else if input.starts_with(PREFIX_OK) {
                let value = strip_prefix_or_err(input, PREFIX_OK)?;
                result.push(RESULT_OK_TAG);
                result.extend(into_bytes(ok, value)?);
                Ok(result)
            } else {
                Err(Error::Formatting(Format::Result))
            }
        }
        NamedCLType::Tuple1(ty) => into_bytes(&ty[0], input),
        NamedCLType::Tuple2(ty) => {
            let parts = input.split(':').collect::<Vec<_>>();
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
            let parts = input.split(':').collect::<Vec<_>>();
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
                    let key_value = part.split('=').collect::<Vec<_>>();
                    if key_value.len() != 2 {
                        return Err(Error::Formatting(Format::Map));
                    }
                    Ok((key_value[0].trim(), key_value[1].trim()))
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
                    let pattern_len = data.len();
                    if pattern_len == 0 {
                        return if n == 0 {
                            Ok(vec![])
                        } else {
                            Err(Error::Formatting(Format::ByteArray))
                        };
                    }

                    if !n.is_multiple_of(pattern_len) {
                        return Err(Error::Formatting(Format::PatternLength {
                            actual: pattern_len,
                            expected: n
                        }));
                    }

                    let count = n / pattern_len;
                    Ok(data.repeat(count))
                }
                Err(Error::InvalidHexString) => {
                    let parts = input.split(',').collect::<Vec<_>>();
                    validate_byte_array_size(n, parts.len())?;

                    if parts.iter().all(|s| s.starts_with(PREFIX_HEX)) {
                        let bytes = parts
                            .iter()
                            .map(|part| parse_hex(part))
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

fn parse_value<T: FromStr>(value: &str) -> TypeResult<T>
where
    <T as FromStr>::Err: Debug
{
    <T as FromStr>::from_str(value).map_err(|err| {
        Error::Parse(format!(
            "Failed to parse value '{}' as {}: {:?}",
            value,
            std::any::type_name::<T>(),
            err
        ))
    })
}

fn parse_hex(input: &str) -> TypeResult<Vec<u8>> {
    if input.is_empty() || input.contains(',') {
        return Err(Error::InvalidHexString);
    }
    if !input.starts_with(PREFIX_HEX) {
        return Err(Error::InvalidHexString);
    }
    match input.strip_prefix(PREFIX_HEX) {
        Some(data) => {
            if data.is_empty() {
                return Err(Error::InvalidHexString);
            }
            if data.len() % 2 != 0 {
                hex::decode(format!("{}{}", data, data)).map_err(|_| Error::HexDecode)
            } else {
                hex::decode(data).map_err(|_| Error::HexDecode)
            }
        }
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
#[inline]
fn strip_prefix_or_err<'a>(input: &'a str, prefix: &str) -> TypeResult<&'a str> {
    input.strip_prefix(prefix).ok_or(Error::Serialization)
}

fn validate_byte_array_size(expected: usize, actual: usize) -> TypeResult<()> {
    if actual != expected {
        return Err(Error::Formatting(Format::InvalidLength {
            actual,
            expected
        }));
    }
    Ok(())
}
