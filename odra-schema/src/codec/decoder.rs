use casper_contract_schema::{CustomType, NamedCLType, Type};
use casper_types::{
    bytesrepr::{FromBytes, RESULT_ERR_TAG, RESULT_OK_TAG},
    Key, PublicKey, URef, U128, U256, U512
};
use serde_json::{Map, Value};

use super::{CustomTypeSet, DecodeError, Error, TypeResult, PREFIX_HEX};

macro_rules! call_from_bytes {
    ($ty:ty, $value:ident) => {
        <$ty as FromBytes>::from_bytes($value)
            .map(|(v, rem)| (v.to_string(), rem))
            .map_err(|_| Error::Serialization)
    };
}

/// Decodes a value of type `ty` from `bytes`, returns it as text and the remaining bytes.
pub fn decode<'a>(
    bytes: &'a [u8],
    ty: &Type,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), DecodeError> {
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

/// Decodes an Odra event (its name followed by its members) as text.
pub fn decode_event(bytes: &[u8], types: &CustomTypeSet) -> Result<String, DecodeError> {
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
) -> Result<(String, &'a [u8]), DecodeError> {
    let matching_type = types
        .iter()
        .find(|t| match t {
            CustomType::Struct { name, .. } => name.0 == ty_name,
            CustomType::Enum { name, .. } => name.0 == ty_name
        })
        .ok_or(DecodeError::UnknownType(ty_name.to_owned()))?;
    let mut bytes = bytes;

    match matching_type {
        CustomType::Struct { members, .. } => {
            let mut object = Map::new();
            for field in members {
                let (value, rem) = decode(bytes, &field.ty, types)?;
                object.insert(field.name.clone(), json_value(&field.ty.0, value));
                bytes = rem;
            }
            Ok((to_pretty_json(&Value::Object(object))?, bytes))
        }
        CustomType::Enum { variants, .. } => {
            let ty = Type(NamedCLType::U8);
            let (value, rem) = decode(bytes, &ty, types)?;
            let discriminant = super::parse_value::<u16>(&value)?;

            let variant = variants
                .iter()
                .find(|v| v.discriminant == discriminant)
                .ok_or(DecodeError::Decoding("Variant not found".to_string()))?;
            bytes = rem;
            Ok((variant.name.clone(), bytes))
        }
    }
}

/// A decoded struct member or list item as a JSON value.
///
/// Nested structs (and lists of them) are already decoded to JSON, so they are embedded as
/// objects. Everything else stays a string, e.g. a `String` member `"123"` is not a number.
fn json_value(ty: &NamedCLType, value: String) -> Value {
    match ty {
        NamedCLType::Custom(_) => from_json_or_string(value),
        NamedCLType::List(item) if matches!(**item, NamedCLType::Custom(_)) => {
            from_json_or_string(value)
        }
        _ => Value::String(value)
    }
}

// An enum decodes to its variant name, which is not JSON.
fn from_json_or_string(value: String) -> Value {
    serde_json::from_str(&value).unwrap_or(Value::String(value))
}

fn decode_list<'a>(
    bytes: &'a [u8],
    inner: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), DecodeError> {
    let ty = Type(inner.clone());
    let (len, mut bytes) = super::from_bytes_or_err::<u32>(bytes)?;
    let mut items = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (value, rem) = decode(bytes, &ty, types)?;
        bytes = rem;
        items.push(value);
    }
    match inner {
        NamedCLType::Custom(_) => {
            let items = items
                .into_iter()
                .map(|item| json_value(inner, item))
                .collect();
            Ok((to_pretty_json(&Value::Array(items))?, bytes))
        }
        _ => Ok((format!("[{}]", items.join(",")), bytes))
    }
}

fn decode_option<'a>(
    bytes: &'a [u8],
    ty: &NamedCLType,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), DecodeError> {
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
) -> Result<(String, &'a [u8]), DecodeError> {
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
        _ => Err(DecodeError::Decoding("Invalid result variant".to_string()))
    }
}

fn decode_tuple1<'a>(
    bytes: &'a [u8],
    types: &[Box<NamedCLType>],
    types_set: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), DecodeError> {
    if types.len() != 1 {
        return Err(DecodeError::Decoding("Invalid tuple length".to_string()));
    }
    let ty = Type(*types[0].clone());
    let (value, rem) = decode(bytes, &ty, types_set)?;
    Ok((format!("({})", value), rem))
}

fn decode_tuple2<'a>(
    bytes: &'a [u8],
    types: &[Box<NamedCLType>],
    types_set: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), DecodeError> {
    if types.len() != 2 {
        return Err(DecodeError::Decoding("Invalid tuple length".to_string()));
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
) -> Result<(String, &'a [u8]), DecodeError> {
    if types.len() != 3 {
        return Err(DecodeError::Decoding("Invalid tuple length".to_string()));
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
) -> Result<(String, &'a [u8]), DecodeError> {
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

fn to_pretty_json(json: &Value) -> Result<String, DecodeError> {
    serde_json::to_string_pretty(json).map_err(|e| DecodeError::Decoding(e.to_string()))
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
        _ => Err(Error::UnexpectedType)
    }
}
