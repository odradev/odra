use std::str::FromStr;

use odra::{
    casper_types::{
        bytesrepr::{FromBytes, RESULT_ERR_TAG, RESULT_OK_TAG},
        Key, PublicKey, URef, U128, U256, U512
    },
    schema::casper_contract_schema::{CustomType, NamedCLType, Type}
};
use serde_json::Value;

use crate::{
    cmd::args::ArgsError,
    custom_types::CustomTypeSet,
    types::{Error, TypeResult, PREFIX_HEX}
};

macro_rules! call_from_bytes {
    ($ty:ty, $value:ident) => {
        <$ty as FromBytes>::from_bytes($value)
            .map(|(v, rem)| (v.to_string(), rem))
            .map_err(|_| Error::Serialization)
    };
}

pub(crate) fn decode<'a>(
    bytes: &'a [u8],
    ty: &Type,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    match &ty.0 {
        NamedCLType::Custom(name) => decode_custom_type(bytes, name, types),
        NamedCLType::List(inner) => decode_list(bytes, inner, types),
        NamedCLType::Option(inner) => decode_option(bytes, inner, types),
        NamedCLType::Result { ok, err } => decode_result(bytes, ok, err, types),
        NamedCLType::Tuple1(ty) => decode_tuple1(bytes, ty, types),
        NamedCLType::Tuple2(ty) => decode_tuple2(bytes, ty, types),
        NamedCLType::Tuple3(ty) => decode_tuple3(bytes, ty, types),
        NamedCLType::Map { key, value } => decode_map(bytes, key, value, types),
        _ => Ok(decode_simple_type(&ty.0, bytes)?)
    }
}

pub(crate) fn decode_event(bytes: &[u8], types: &CustomTypeSet) -> Result<String, ArgsError> {
    // Event name is stored as the first element in the bytes
    let (mut name, rem): (String, _) =
        FromBytes::from_bytes(bytes).map_err(|_| Error::InvalidEventType("Unknown".to_string()))?;
    let mut bytes = rem;
    // Ignore the `event_` prefix
    let event_name = name.split_off(6);
    let members = types
        .iter()
        .find_map(|ty| match ty {
            CustomType::Struct { name, members, .. } if name.0 == event_name => Some(members),
            _ => None
        })
        .ok_or_else(|| Error::InvalidEventType(event_name.clone()))?;

    let mut output = format!("'{}':\n", event_name);
    for m in members {
        let (data, rem) = decode(bytes, &m.ty, types)?;
        bytes = rem;
        output.push_str(&format!("  '{}': {}\n", m.name, data));
    }
    Ok(output)
}

fn decode_custom_type<'a>(
    bytes: &'a [u8],
    ty_name: &str,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    let matching_type = types
        .iter()
        .find(|t| match t {
            CustomType::Struct { name, .. } => name.0 == ty_name,
            CustomType::Enum { name, .. } => name.0 == ty_name
        })
        .ok_or(ArgsError::ArgTypeNotFound(ty_name.to_owned()))?;
    let mut bytes = bytes;

    match matching_type {
        CustomType::Struct { members, .. } => {
            let mut decoded = "{ ".to_string();
            for field in members {
                let (value, rem) = decode(bytes, &field.ty, types)?;
                decoded.push_str(format!(" \"{}\": \"{}\",", field.name, value).as_str());
                bytes = rem;
            }
            decoded.pop();
            decoded.push_str(" }");
            Ok((to_json(&decoded)?, bytes))
        }
        CustomType::Enum { variants, .. } => {
            let ty = Type(NamedCLType::U8);
            let (value, rem) = decode(bytes, &ty, types)?;
            let discriminant = super::parse_value::<u16>(&value)?;

            let variant = variants
                .iter()
                .find(|v| v.discriminant == discriminant)
                .ok_or(ArgsError::DecodingError("Variant not found".to_string()))?;
            bytes = rem;
            Ok((variant.name.clone(), bytes))
        }
    }
}

fn decode_list<'a>(
    bytes: &'a [u8],
    inner: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    let ty = Type(inner.clone());
    let mut bytes = bytes;
    let mut decoded = "[".to_string();

    let (len, rem) = super::from_bytes_or_err::<u32>(bytes)?;
    bytes = rem;
    for _ in 0..len {
        let (value, rem) = decode(bytes, &ty, types)?;
        bytes = rem;
        decoded.push_str(format!("{},", value).as_str());
    }
    if len > 0 {
        decoded.pop(); // remove trailing comma
    }
    decoded.push(']');
    match inner {
        NamedCLType::Custom(_) => Ok((to_json(&decoded)?, bytes)),
        _ => Ok((decoded, bytes))
    }
}

fn decode_option<'a>(
    bytes: &'a [u8],
    ty: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    let (is_some, rem) = super::from_bytes_or_err::<bool>(bytes)?;
    if is_some {
        let ty = Type(ty.clone());
        let (value, rem) = decode(rem, &ty, types)?;
        Ok((value, rem))
    } else {
        Ok(("None".to_string(), rem))
    }
}

fn decode_result<'a>(
    bytes: &'a [u8],
    ok: &NamedCLType,
    err: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    let (variant, rem) = super::from_bytes_or_err::<u8>(bytes)?;
    match variant {
        RESULT_ERR_TAG => {
            let ty = Type(err.clone());
            let (value, rem) = decode(rem, &ty, types)?;
            Ok((format!("Err({})", value), rem))
        }
        RESULT_OK_TAG => {
            let ty = Type(ok.clone());
            let (value, rem) = decode(rem, &ty, types)?;
            Ok((format!("Ok({})", value), rem))
        }
        _ => Err(ArgsError::DecodingError(
            "Invalid result variant".to_string()
        ))
    }
}

fn decode_tuple1<'a>(
    bytes: &'a [u8],
    types: &[Box<NamedCLType>],
    types_set: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    if types.len() != 1 {
        return Err(ArgsError::DecodingError("Invalid tuple length".to_string()));
    }
    let ty = Type(*types[0].clone());
    let (value, rem) = decode(bytes, &ty, types_set)?;
    Ok((format!("({})", value), rem))
}

fn decode_tuple2<'a>(
    bytes: &'a [u8],
    types: &[Box<NamedCLType>],
    types_set: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    if types.len() != 2 {
        return Err(ArgsError::DecodingError("Invalid tuple length".to_string()));
    }
    let ty1 = Type(*types[0].clone());
    let ty2 = Type(*types[1].clone());
    let (v1, rem) = decode(bytes, &ty1, types_set)?;
    let (v2, rem) = decode(rem, &ty2, types_set)?;
    Ok((format!("({}, {})", v1, v2), rem))
}

fn decode_tuple3<'a>(
    bytes: &'a [u8],
    types: &[Box<NamedCLType>],
    types_set: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    if types.len() != 3 {
        return Err(ArgsError::DecodingError("Invalid tuple length".to_string()));
    }
    let ty1 = Type(*types[0].clone());
    let ty2 = Type(*types[1].clone());
    let ty3 = Type(*types[2].clone());
    let (v1, rem) = decode(bytes, &ty1, types_set)?;
    let (v2, rem) = decode(rem, &ty2, types_set)?;
    let (v3, rem) = decode(rem, &ty3, types_set)?;
    Ok((format!("({}, {}, {})", v1, v2, v3), rem))
}

fn decode_map<'a>(
    bytes: &'a [u8],
    key: &NamedCLType,
    value: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    let (num_keys, mut stream) = super::from_bytes_or_err::<u32>(bytes)?;
    let mut result = String::new();
    for _ in 0..num_keys {
        let k_ty = Type(key.clone());
        let v_ty = Type(value.clone());
        let (k, rem) = decode(stream, &k_ty, types)?;
        let (v, rem) = decode(rem, &v_ty, types)?;
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

fn to_json(str: &str) -> Result<String, ArgsError> {
    let json =
        Value::from_str(str).map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))?;
    serde_json::to_string_pretty(&json)
        .map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))
}

fn decode_simple_type<'a>(ty: &NamedCLType, input: &'a [u8]) -> TypeResult<(String, &'a [u8])> {
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
        NamedCLType::Unit => <() as FromBytes>::from_bytes(input)
            .map(|(_, rem)| ("".to_string(), rem))
            .map_err(|_| Error::Deserialization),
        NamedCLType::ByteArray(n) => {
            let size = *n as usize;
            let mut hex = PREFIX_HEX.to_string();
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
        _ => unreachable!("should not be here")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use odra::{
        casper_types::bytesrepr::{ToBytes, RESULT_ERR_TAG, RESULT_OK_TAG},
        schema::casper_contract_schema::{NamedCLType, Type}
    };

    use crate::test_utils;

    const NAMED_TOKEN_METADATA_BYTES: [u8; 50] = [
        4, 0, 0, 0, 107, 112, 111, 98, 0, 32, 74, 169, 209, 1, 0, 0, 1, 1, 226, 74, 54, 110, 186,
        196, 135, 233, 243, 218, 49, 175, 91, 142, 42, 103, 172, 205, 97, 76, 95, 247, 61, 188, 60,
        100, 10, 52, 124, 59, 94, 73
    ];

    const NAMED_TOKEN_METADATA_JSON: &str = r#"{
  "token_hash": "kpob",
  "expiration": "2000000000000",
  "resolver": "Key::Hash(e24a366ebac487e9f3da31af5b8e2a67accd614c5ff73dbc3c640a347c3b5e49)"
}"#;

    #[test]
    fn test_decode_custom_type() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Custom("NameTokenMetadata".to_string()));
        let (result, _bytes) =
            super::decode(&NAMED_TOKEN_METADATA_BYTES, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, NAMED_TOKEN_METADATA_JSON);
    }

    #[test]
    fn test_decode_map() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Map {
            key: Box::new(NamedCLType::String),
            value: Box::new(NamedCLType::U64)
        });

        let map = BTreeMap::from_iter([("foo".to_string(), 1u64), ("bar".to_string(), 2u64)]);
        let bytes = map.to_bytes().unwrap();

        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, r#"bar:2, foo:1"#);
    }

    #[test]
    fn test_decode_list() {
        let custom_types = test_utils::custom_types();
        let ty = Type(NamedCLType::List(Box::new(NamedCLType::U64)));
        let list = vec![1u64, 2u64, 3u64];
        let bytes = list.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "[1,2,3]");

        let list = Vec::<u64>::new();
        let bytes = list.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "[]");
    }

    #[test]
    fn test_decode_option() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Option(Box::new(NamedCLType::U64)));
        let value = Some(42u64);
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "42");

        let value: Option<u64> = None;
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "None");
    }

    #[test]
    fn test_decode_result() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Result {
            ok: Box::new(NamedCLType::U64),
            err: Box::new(NamedCLType::String)
        });

        let value: Result<u64, String> = Ok(42u64);
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "Ok(42)");

        let value: Result<u64, String> = Err("Error".to_string());
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "Err(Error)");
    }

    #[test]
    fn test_decode_tuple() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Tuple1([Box::new(NamedCLType::U64)]));

        let value = (42u64,);
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "(42)");

        let ty = Type(NamedCLType::Tuple2([
            Box::new(NamedCLType::U64),
            Box::new(NamedCLType::String)
        ]));

        let value = (42u64, "Hello".to_string());
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "(42, Hello)");

        let ty = Type(NamedCLType::Tuple3([
            Box::new(NamedCLType::U64),
            Box::new(NamedCLType::String),
            Box::new(NamedCLType::Bool)
        ]));

        let value = (42u64, "Hello".to_string(), true);
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "(42, Hello, true)");
    }

    #[test]
    fn test_option_custom_type() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Option(Box::new(NamedCLType::Custom(
            "NameTokenMetadata".to_string()
        ))));

        let mut bytes = vec![1];
        bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, NAMED_TOKEN_METADATA_JSON);

        let (result, _bytes) = super::decode(&[0], &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "None");
    }

    #[test]
    fn test_decode_simple_type() {
        let custom_types = test_utils::custom_types();
        let ty = Type(NamedCLType::U64);
        let value = 42u64;
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "42");

        let ty = Type(NamedCLType::String);
        let value = "Hello".to_string();
        let bytes = value.to_bytes().unwrap();
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "Hello");
    }

    #[test]
    fn test_decode_result_custom_type() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Result {
            ok: Box::new(NamedCLType::Custom("NameTokenMetadata".to_string())),
            err: Box::new(NamedCLType::String)
        });

        let mut bytes = vec![RESULT_OK_TAG];
        bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, format!("Ok({})", NAMED_TOKEN_METADATA_JSON));

        let mut bytes = vec![RESULT_ERR_TAG];
        bytes.extend_from_slice(&"Error".to_string().to_bytes().unwrap());
        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, "Err(Error)");
    }

    #[test]
    fn test_decode_map_custom_type() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Map {
            key: Box::new(NamedCLType::String),
            value: Box::new(NamedCLType::Custom("NameTokenMetadata".to_string()))
        });

        let mut bytes = 1u32.to_bytes().unwrap();
        bytes.extend_from_slice(&"foo".to_string().to_bytes().unwrap());
        bytes.extend_from_slice(&NAMED_TOKEN_METADATA_BYTES);

        let (result, _bytes) = super::decode(&bytes, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, format!("foo:{}", NAMED_TOKEN_METADATA_JSON));
    }
}
