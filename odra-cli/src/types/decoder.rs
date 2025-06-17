use std::str::FromStr;

use odra::schema::casper_contract_schema::{CustomType, NamedCLType, Type};
use serde_json::Value;

use crate::{cmd::args::ArgsError, custom_types::CustomTypeSet};

pub fn decode<'a>(
    bytes: &'a [u8],
    ty: &Type,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    match &ty.0 {
        NamedCLType::Custom(name) => {
            let matching_type = types
                .iter()
                .find(|ty| {
                    let type_name = match ty {
                        CustomType::Struct { name, .. } => &name.0,
                        CustomType::Enum { name, .. } => &name.0
                    };
                    name == type_name
                })
                .ok_or(ArgsError::ArgTypeNotFound(name.clone()))?;
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
        NamedCLType::List(inner) => {
            let ty = Type(*inner.clone());
            let mut bytes = bytes;
            let mut decoded = "[".to_string();

            let (len, rem) = super::from_bytes_or_err::<u32>(bytes)?;
            bytes = rem;
            for _ in 0..len {
                let (value, rem) = decode(bytes, &ty, types)?;
                bytes = rem;
                decoded.push_str(format!("{},", value).as_str());
            }
            decoded.pop();
            decoded.push(']');
            match inner.as_ref() {
                NamedCLType::Custom(_) => Ok((to_json(&decoded)?, bytes)),
                _ => Ok((decoded, bytes))
            }
        }
        _ => {
            let result = super::from_bytes(&ty.0, bytes)?;
            Ok(result)
        }
    }
}

fn to_json(str: &str) -> Result<String, ArgsError> {
    let json =
        Value::from_str(str).map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))?;
    serde_json::to_string_pretty(&json)
        .map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))
}

#[cfg(test)]
mod tests {
    use odra::schema::casper_contract_schema::{NamedCLType, Type};

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
    fn test_decode() {
        let custom_types = test_utils::custom_types();

        let ty = Type(NamedCLType::Custom("NameTokenMetadata".to_string()));
        let (result, _bytes) =
            super::decode(&NAMED_TOKEN_METADATA_BYTES, &ty, &custom_types).unwrap();
        pretty_assertions::assert_eq!(result, NAMED_TOKEN_METADATA_JSON);
    }
}
