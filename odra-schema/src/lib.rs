use std::{collections::BTreeSet, env};

pub use casper_contract_schema;
use casper_contract_schema::{
    Access, Argument, CallMethod, ContractSchema, CustomType, Entrypoint, EnumVariant, Event,
    NamedCLType, StructMember
};

use odra_core::args::EntrypointArgument;

const CCSV: u8 = 1;

mod custom_type;
mod ty;

pub use custom_type::SchemaCustomType;
pub use ty::NamedCLTyped;

pub trait SchemaEntrypoints {
    fn schema_entrypoints() -> Vec<Entrypoint>;
}

pub trait SchemaEvents {
    fn schema_events() -> Vec<Event>;
}

pub trait SchemaCustomTypes {
    fn schema_types() -> Vec<Option<CustomType>>;
}

pub fn argument<T: NamedCLTyped + EntrypointArgument>(name: &str) -> Argument {
    if T::is_required() {
        Argument::new(name, "", <T as NamedCLTyped>::ty())
    } else {
        Argument::new_opt(name, "", <T as NamedCLTyped>::ty())
    }
}

pub fn entry_point<T: NamedCLTyped>(
    name: &str,
    description: &str,
    is_mutable: bool,
    arguments: Vec<Argument>
) -> Entrypoint {
    Entrypoint {
        name: name.into(),
        description: Some(description.to_string()),
        is_mutable,
        arguments,
        return_ty: T::ty().into(),
        is_contract_context: true,
        access: Access::Public
    }
}

pub fn struct_member<T: NamedCLTyped>(name: &str) -> StructMember {
    StructMember {
        name: name.to_string(),
        description: None,
        ty: T::ty().into()
    }
}

pub fn enum_typed_variant<T: NamedCLTyped>(name: &str, discriminant: u8) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: T::ty().into()
    }
}

pub fn enum_variant(name: &str, discriminant: u8) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: NamedCLType::Unit.into()
    }
}

pub fn custom_struct(name: &str, members: Vec<StructMember>) -> CustomType {
    CustomType::Struct {
        name: name.into(),
        description: None,
        members
    }
}

pub fn custom_enum(name: &str, variants: Vec<EnumVariant>) -> CustomType {
    CustomType::Enum {
        name: name.into(),
        description: None,
        variants
    }
}

pub fn event(name: &str) -> Event {
    Event {
        name: name.into(),
        ty: name.into()
    }
}

pub fn schema<T: SchemaEntrypoints + SchemaEvents + SchemaCustomTypes>(
    module_name: &str,
    contract_name: &str,
    contract_version: &str,
    authors: Vec<String>,
    repository: &str,
    homepage: &str
) -> ContractSchema {
    let entry_points = T::schema_entrypoints();
    let events = T::schema_events();
    let types = BTreeSet::from_iter(T::schema_types())
        .into_iter()
        .flatten()
        .collect();

    let init_ep = entry_points.iter().find(|e| e.name == "init");

    let init_args = init_ep.map(|e| e.arguments.clone()).unwrap_or_default();

    let init_description = init_ep.and_then(|e| e.description.clone());

    let entry_points = entry_points
        .into_iter()
        .filter(|e| e.name != "init")
        .collect();

    let wasm_file_name = format!("{}.wasm", module_name);

    let repository = match repository {
        "" => None,
        _ => Some(repository.to_string())
    };

    let homepage = match homepage {
        "" => None,
        _ => Some(homepage.to_string())
    };

    ContractSchema {
        casper_contract_schema_version: CCSV,
        toolchain: env!("RUSTC_VERSION").to_string(),
        contract_name: contract_name.to_string(),
        contract_version: contract_version.to_string(),
        types,
        entry_points,
        events,
        call: Some(call_method(wasm_file_name, init_description, &init_args)),
        authors,
        repository,
        homepage,
        errors: vec![]
    }
}

fn call_method(
    file_name: String,
    description: Option<String>,
    constructor_args: &[Argument]
) -> CallMethod {
    CallMethod {
        wasm_file_name: file_name.to_string(),
        description: description.map(String::from),
        arguments: vec![
            Argument {
                name: odra_core::consts::PACKAGE_HASH_KEY_NAME_ARG.to_string(),
                description: Some("The arg name for the package hash key name.".to_string()),
                ty: NamedCLType::String.into(),
                optional: false
            },
            Argument {
                name: odra_core::consts::ALLOW_KEY_OVERRIDE_ARG.to_string(),
                description: Some("The arg name for the allow key override.".to_string()),
                ty: NamedCLType::Bool.into(),
                optional: false
            },
            Argument {
                name: odra_core::consts::IS_UPGRADABLE_ARG.to_string(),
                description: Some(
                    "The arg name for the contract upgradeability setting.".to_string()
                ),
                ty: NamedCLType::Bool.into(),
                optional: false
            },
        ]
        .iter()
        .chain(constructor_args.iter())
        .cloned()
        .collect()
    }
}
