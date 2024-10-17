use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches};
use odra::casper_types::{CLType, CLValue, RuntimeArgs};
use odra::schema::casper_contract_schema::{Argument, CustomType, Entrypoint, NamedCLType, Type};
use serde_json::Value;
use thiserror::Error;

use crate::{types, CustomTypeSet};

pub const ARG_ATTACHED_VALUE: &str = "__attached_value";

#[derive(Debug, Error)]
pub enum ArgsError {
    #[error("Invalid arg value: {0}")]
    TypesError(#[from] types::Error),
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Arg not found: {0}")]
    ArgNotFound(String),
    #[error("Arg type not found: {0}")]
    ArgTypeNotFound(String)
}

/// A typed command argument.
#[derive(Debug, PartialEq)]
pub struct CommandArg {
    pub name: String,
    pub required: bool,
    pub description: String,
    pub ty: NamedCLType,
    pub is_list_element: bool
}

impl CommandArg {
    pub fn new(
        name: &str,
        description: &str,
        ty: NamedCLType,
        required: bool,
        is_list_element: bool
    ) -> Self {
        Self {
            name: name.to_string(),
            required,
            description: description.to_string(),
            ty,
            is_list_element
        }
    }
}

impl From<CommandArg> for Arg {
    fn from(arg: CommandArg) -> Self {
        let result = Arg::new(&arg.name)
            .long(arg.name)
            .value_name(format!("{:?}", arg.ty))
            .required(arg.required)
            .help(arg.description);

        match arg.is_list_element {
            true => result.action(ArgAction::Append),
            false => result.action(ArgAction::Set)
        }
    }
}

pub fn entry_point_args(entry_point: &Entrypoint, types: &CustomTypeSet) -> Vec<Arg> {
    entry_point
        .arguments
        .iter()
        .flat_map(|arg| flat_arg(arg, types, false))
        .flatten()
        .map(Into::into)
        .collect()
}

fn flat_arg(
    arg: &Argument,
    types: &CustomTypeSet,
    is_list_element: bool
) -> Result<Vec<CommandArg>, ArgsError> {
    match &arg.ty.0 {
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

            match matching_type {
                CustomType::Struct { members, .. } => {
                    let commands = members
                        .iter()
                        .map(|field| {
                            let field_arg = Argument {
                                name: format!("{}.{}", arg.name, field.name),
                                ty: field.ty.clone(),
                                optional: arg.optional,
                                description: field.description.clone()
                            };
                            flat_arg(&field_arg, types, is_list_element)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(commands.into_iter().flatten().collect())
                }
                CustomType::Enum { variants, .. } => {
                    let commands = variants
                        .iter()
                        .map(|variant| {
                            let variant_arg = Argument {
                                name: format!("{}.{}", arg.name, variant.name.to_lowercase()),
                                ty: variant.ty.clone(),
                                optional: arg.optional,
                                description: variant.description.clone()
                            };
                            flat_arg(&variant_arg, types, is_list_element)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(commands.into_iter().flatten().collect())
                }
            }
        }
        NamedCLType::List(inner) => {
            let arg = Argument {
                ty: Type(*inner.clone()),
                ..arg.clone()
            };
            flat_arg(&arg, types, true)
        }
        _ => Ok(vec![CommandArg::new(
            &arg.name,
            &arg.description.clone().unwrap_or_default(),
            arg.ty.0.clone(),
            !arg.optional,
            is_list_element
        )])
    }
}

pub fn compose(
    entry_point: &Entrypoint,
    args: &ArgMatches,
    types: &CustomTypeSet
) -> Result<RuntimeArgs, ArgsError> {
    let mut runtime_args = RuntimeArgs::new();

    for arg in entry_point.arguments.iter() {
        let parts: Vec<CommandArg> = flat_arg(arg, types, false)?;

        let cl_value = if parts.len() == 1 {
            let input = args
                .get_many::<String>(&arg.name)
                .unwrap_or_default()
                .map(|s| s.as_str())
                .collect::<Vec<_>>();
            let ty = &arg.ty.0;
            if input.is_empty() {
                continue;
            }
            match ty {
                NamedCLType::List(inner) => {
                    let input = input
                        .iter()
                        .flat_map(|v| v.split(',').collect::<Vec<_>>())
                        .collect();
                    let bytes = types::vec_into_bytes(inner, input)?;
                    let cl_type = CLType::List(Box::new(types::named_cl_type_to_cl_type(inner)));
                    CLValue::from_components(cl_type, bytes)
                }
                _ => {
                    let bytes = types::into_bytes(ty, input[0])?;
                    let cl_type = types::named_cl_type_to_cl_type(ty);
                    CLValue::from_components(cl_type, bytes)
                }
            }
        } else {
            build_complex_arg(parts, args)?
        };
        runtime_args.insert_cl_value(arg.name.clone(), cl_value);
    }

    Ok(runtime_args)
}

#[derive(Debug, PartialEq)]
struct ComposedArg<'a> {
    name: String,
    values: Vec<Values<'a>>
}
type Values<'a> = (NamedCLType, Vec<&'a str>);

impl<'a> ComposedArg<'a> {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: vec![]
        }
    }

    fn add(&mut self, value: Values<'a>) {
        self.values.push(value);
    }

    fn flush(&mut self, buffer: &mut Vec<u8>) -> Result<(), ArgsError> {
        if self.values.is_empty() {
            return Ok(());
        }
        let size = self.values[0].1.len();

        // check if all values have the same length
        let equals_len = self
            .values
            .iter()
            .map(|(_, vec)| vec.len())
            .all(|len| len == size);

        if !equals_len {
            return Err(ArgsError::DecodingError(format!(
                "Not equal args length for the list `{}`",
                self.name
            )));
        }

        buffer.extend(types::to_bytes_or_err(size as u32)?);

        for i in 0..size {
            for (ty, values) in &self.values {
                let bytes = types::into_bytes(ty, values[i])?;
                buffer.extend_from_slice(&bytes);
            }
        }
        self.values.clear();
        Ok(())
    }
}

fn build_complex_arg(args: Vec<CommandArg>, matches: &ArgMatches) -> Result<CLValue, ArgsError> {
    let mut current_group = ComposedArg::new("");
    let mut buffer: Vec<u8> = vec![];
    for arg in args {
        let args = matches
            .get_many::<String>(&arg.name)
            .ok_or(ArgsError::ArgNotFound(arg.name.clone()))?
            .map(|v| v.as_str())
            .collect::<Vec<_>>();
        let ty = arg.ty;
        let is_list_element = arg.is_list_element;

        let parts = arg
            .name
            .split('.')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let parent = parts[parts.len() - 2].clone();

        if current_group.name != parent && is_list_element {
            current_group.flush(&mut buffer)?;
            current_group = ComposedArg::new(&parent);
            current_group.add((ty, args));
        } else if current_group.name == parent && is_list_element {
            current_group.add((ty, args));
        } else {
            current_group.flush(&mut buffer)?;
            let bytes = types::into_bytes(&ty, args[0])?;
            buffer.extend_from_slice(&bytes);
        }
    }
    current_group.flush(&mut buffer)?;
    Ok(CLValue::from_components(CLType::Any, buffer))
}

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
                    let discriminant = types::parse_value::<u16>(&value)?;

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

            let (len, rem) = types::from_bytes_or_err::<u32>(bytes)?;
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
            let result = types::from_bytes(&ty.0, bytes)?;
            Ok(result)
        }
    }
}

fn to_json(str: &str) -> Result<String, ArgsError> {
    let json = Value::from_str(str)
        .map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))?;
    serde_json::to_string_pretty(&json)
        .map_err(|_| ArgsError::DecodingError("Invalid JSON".to_string()))
}

pub fn attached_value_arg() -> Arg {
    Arg::new(ARG_ATTACHED_VALUE)
        .help("The amount of CSPRs attached to the call")
        .long(ARG_ATTACHED_VALUE)
        .required(false)
        .value_name(format!("{:?}", NamedCLType::U512))
        .action(ArgAction::Set)
}

#[cfg(test)]
mod tests {
    use clap::{Arg, Command};
    use odra::casper_types::{bytesrepr::Bytes, runtime_args, RuntimeArgs};
    use odra::schema::casper_contract_schema::{NamedCLType, Type};

    use crate::test_utils::{self, NameMintInfo, PaymentInfo, PaymentVoucher};

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

    #[test]
    fn test_command_args() {
        let entry_point = test_utils::mock_entry_point();
        let custom_types = test_utils::custom_types();

        let args = entry_point
            .arguments
            .iter()
            .flat_map(|arg| super::flat_arg(arg, &custom_types, false))
            .flatten()
            .collect::<Vec<_>>();

        let expected = test_utils::mock_command_args();
        pretty_assertions::assert_eq!(args, expected);
    }

    #[test]
    fn test_compose() {
        let entry_point = test_utils::mock_entry_point();

        let mut cmd = Command::new("myprog");
        test_utils::mock_command_args()
            .into_iter()
            .map(Into::into)
            .for_each(|arg: Arg| {
                cmd = cmd.clone().arg(arg);
            });

        let args = cmd.get_matches_from(vec![
            "myprog",
            "--voucher.payment.buyer",
            "hash-56fef1f62d86ab68655c2a5d1c8b9ed8e60d5f7e59736e9d4c215a40b10f4a22",
            "--voucher.payment.payment_id",
            "id_001",
            "--voucher.payment.amount",
            "666",
            "--voucher.names.label",
            "kpob",
            "--voucher.names.owner",
            "hash-f01cec215ddfd4c4a19d58f9c917023391a1da871e047dc47a83ae55f6cfc20a",
            "--voucher.names.token_expiration",
            "1000000",
            "--voucher.names.label",
            "qwerty",
            "--voucher.names.owner",
            "hash-f01cec215ddfd4c4a19d58f9c917023391a1da871e047dc47a83ae55f6cfc20a",
            "--voucher.names.token_expiration",
            "1000000",
            "--voucher.voucher_expiration",
            "2000000",
            "--signature",
            "1,148,81,107,136,16,186,87,48,202,151",
        ]);
        let types = test_utils::custom_types();
        let args = super::compose(&entry_point, &args, &types).unwrap();
        let expected = runtime_args! {
            "voucher" => PaymentVoucher::new(
                PaymentInfo::new(
                    "hash-56fef1f62d86ab68655c2a5d1c8b9ed8e60d5f7e59736e9d4c215a40b10f4a22",
                    "id_001",
                    "666"
                ),
                vec![
                    NameMintInfo::new(
                        "kpob",
                        "hash-f01cec215ddfd4c4a19d58f9c917023391a1da871e047dc47a83ae55f6cfc20a",
                        1000000
                    ),
                    NameMintInfo::new(
                        "qwerty",
                        "hash-f01cec215ddfd4c4a19d58f9c917023391a1da871e047dc47a83ae55f6cfc20a",
                        1000000
                    )
                ],
                2000000
            ),
            "signature" => Bytes::from(vec![1u8, 148u8, 81u8, 107u8, 136u8, 16u8, 186u8, 87u8, 48u8, 202u8, 151u8]),
        };
        pretty_assertions::assert_eq!(args, expected);
    }
}
