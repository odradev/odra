use clap::ArgMatches;
use odra::casper_types::{CLType, CLValue, RuntimeArgs};
use odra::schema::casper_contract_schema::{Argument, Entrypoint, NamedCLType};

use crate::cmd::args::{ArgsError, CommandArg};
use crate::custom_types::CustomTypeSet;
use crate::entry_point::utils::flatten_schema_arg;
use crate::types;

pub fn compose(
    entry_point: &Entrypoint,
    args: &ArgMatches,
    types: &CustomTypeSet
) -> Result<RuntimeArgs, ArgsError> {
    let mut runtime_args = RuntimeArgs::new();

    for arg in entry_point.arguments.iter() {
        let parts: Vec<CommandArg> = flatten_schema_arg(arg, types, false)?;
        let cl_value = if parts.len() == 1 {
            compose_basic_arg(arg, args)?
        } else {
            compose_complex_arg(parts, args)?
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
type Values<'a> = Vec<&'a CLValue>;

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
        let size = self.values[0].len();

        // check if all values have the same length
        let equals_len = self
            .values
            .iter()
            .map(|vec| vec.len())
            .all(|len| len == size);

        if !equals_len {
            return Err(ArgsError::DecodingError(format!(
                "Not equal args length for the list `{}`",
                self.name
            )));
        }

        buffer.extend(types::to_bytes_or_err(size as u32)?);

        for i in 0..size {
            for values in &self.values {
                buffer.extend_from_slice(values[i].inner_bytes());
            }
        }
        self.values.clear();
        Ok(())
    }
}

fn compose_complex_arg(args: Vec<CommandArg>, matches: &ArgMatches) -> Result<CLValue, ArgsError> {
    let mut current_group = ComposedArg::new("");
    let mut buffer: Vec<u8> = vec![];
    for arg in args {
        let cl_values = matches
            .get_many::<CLValue>(&arg.name)
            .ok_or(ArgsError::ArgNotFound(arg.name.clone()))?
            .collect::<Vec<_>>();
        let is_list_element = arg.is_list_element;

        let parts = arg.split_name();
        let parent = parts[parts.len() - 2].clone();

        if current_group.name != parent && is_list_element {
            current_group.flush(&mut buffer)?;
            current_group = ComposedArg::new(&parent);
            current_group.add(cl_values);
        } else if current_group.name == parent && is_list_element {
            current_group.add(cl_values);
        } else {
            current_group.flush(&mut buffer)?;
            buffer.extend_from_slice(cl_values[0].inner_bytes());
        }
    }
    current_group.flush(&mut buffer)?;
    Ok(CLValue::from_components(CLType::Any, buffer))
}

fn compose_basic_arg(arg: &Argument, matches: &ArgMatches) -> Result<CLValue, ArgsError> {
    let input = matches
        .get_many::<CLValue>(&arg.name)
        .unwrap_or_default()
        .collect::<Vec<_>>();

    if input.is_empty() {
        return Err(ArgsError::ArgNotFound(arg.name.clone()));
    }

    Ok(match &arg.ty.0 {
        NamedCLType::List(box NamedCLType::U8) => input[0].to_owned(),
        NamedCLType::List(inner) => {
            let mut bytes = types::to_bytes_or_err(input.len() as u32)?;
            for value in input {
                bytes.extend(value.inner_bytes());
            }
            let cl_type = CLType::List(Box::new(types::named_cl_type_to_cl_type(inner)));
            CLValue::from_components(cl_type, bytes)
        }
        _ => input[0].to_owned()
    })
}

#[cfg(test)]
mod tests {
    use clap::Command;
    use odra::casper_types::{bytesrepr::Bytes, runtime_args};

    use crate::test_utils::{self, NameMintInfo, PaymentInfo, PaymentVoucher};

    #[test]
    fn test_compose() {
        let entry_point = test_utils::mock_entry_point();
        let cmd = Command::new("myprog").args(test_utils::mock_command_args());

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
