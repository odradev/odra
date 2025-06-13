use odra::schema::casper_contract_schema::{Argument, CustomType, NamedCLType, Type};

use crate::cmd::args::{ArgsError, CommandArg};
use crate::CustomTypeSet;

pub(super) fn flatten_schema_arg(
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
                            flatten_schema_arg(&field_arg, types, is_list_element)
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
                            flatten_schema_arg(&variant_arg, types, is_list_element)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(commands.into_iter().flatten().collect())
                }
            }
        }
        NamedCLType::List(box NamedCLType::U8) => {
            let mut ca = CommandArg::new(
                &arg.name,
                &arg.description.clone().unwrap_or_default(),
                NamedCLType::List(Box::new(NamedCLType::U8))
            )
            .list();
            if !arg.optional {
                ca = ca.required();
            }
            Ok(vec![ca])
        }
        NamedCLType::List(inner) => {
            let arg = Argument {
                ty: Type(*inner.clone()),
                ..arg.clone()
            };
            flatten_schema_arg(&arg, types, true)
        }
        _ => {
            let mut ca = CommandArg::new(
                &arg.name,
                &arg.description.clone().unwrap_or_default(),
                arg.ty.0.clone()
            );
            if !arg.optional {
                ca = ca.required();
            }
            if is_list_element {
                ca = ca.list();
            }
            Ok(vec![ca])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    #[test]
    fn test_command_args() {
        let entry_point = test_utils::mock_entry_point();
        let custom_types = test_utils::custom_types();

        let args = entry_point
            .arguments
            .iter()
            .flat_map(|arg| super::flatten_schema_arg(arg, &custom_types, false))
            .flatten()
            .collect::<Vec<_>>();
        let expected = test_utils::mock_command_args();
        pretty_assertions::assert_eq!(expected, args);
    }
}
