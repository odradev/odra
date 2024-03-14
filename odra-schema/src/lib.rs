use std::collections::BTreeSet;

pub use casper_contract_schema;
use casper_contract_schema::{
    Access, Argument, CallMethod, ContractSchema, CustomType, Entrypoint, EnumVariant, Event,
    StructMember, Type
};
use casper_types::CLType;
use odra_core::args::EntrypointArgument;

const CCSV: u8 = 1;

mod custom_type;
mod element;

pub use custom_type::SchemaCustomType;
pub use element::SchemaElement;

pub trait SchemaEntrypoints {
    fn schema_entrypoints() -> Vec<Entrypoint>;
}

pub trait SchemaEvents {
    fn schema_events() -> Vec<Event>;
}

pub trait SchemaCustomTypes {
    fn schema_types() -> Vec<Option<CustomType>>;
}

pub fn argument<T: SchemaElement + EntrypointArgument>(name: &str) -> Argument {
    Argument::new(name, "", <T as SchemaElement>::ty(), !T::is_required())
}

pub fn entry_point<T: SchemaElement>(
    name: &str,
    is_mutable: bool,
    arguments: Vec<Argument>
) -> Entrypoint {
    Entrypoint {
        name: name.into(),
        description: None,
        is_mutable,
        arguments,
        return_ty: T::ty(),
        is_contract_context: true,
        access: Access::Public
    }
}

pub fn struct_member<T: SchemaElement>(name: &str) -> StructMember {
    StructMember {
        name: name.to_string(),
        description: None,
        ty: T::ty()
    }
}

pub fn enum_typed_variant<T: SchemaElement>(name: &str, discriminant: u8) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: Some(T::ty())
    }
}

pub fn enum_variant(name: &str, discriminant: u8) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: None
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
    contract_version: &str
) -> ContractSchema {
    let entry_points = T::schema_entrypoints();
    let events = T::schema_events();
    let types = BTreeSet::from_iter(T::schema_types())
        .into_iter()
        .flatten()
        .collect();

    let init_args = entry_points
        .iter()
        .find(|e| e.name == "init")
        .map(|e| e.arguments.clone())
        .unwrap_or_default();

    let entry_points = entry_points
        .into_iter()
        .filter(|e| e.name != "init")
        .collect();

    let wasm_file_name = format!("{}.wasm", module_name);

    ContractSchema {
        casper_contract_schema_version: CCSV,
        toolchain: env!("RUSTC_VERSION").to_string(),
        contract_name: contract_name.to_string(),
        contract_version: contract_version.to_string(),
        types,
        entry_points,
        events,
        call: Some(call_method(wasm_file_name, None, &init_args))
    }
}

fn call_method(
    file_name: String,
    description: Option<&str>,
    constructor_args: &[Argument]
) -> CallMethod {
    CallMethod {
        wasm_file_name: file_name.to_string(),
        description: description.map(String::from),
        arguments: vec![
            Argument {
                name: odra_core::consts::PACKAGE_HASH_KEY_NAME_ARG.to_string(),
                description: None,
                ty: Type::System(CLType::String),
                optional: false
            },
            Argument {
                name: odra_core::consts::ALLOW_KEY_OVERRIDE_ARG.to_string(),
                description: None,
                ty: Type::System(CLType::Bool),
                optional: false
            },
            Argument {
                name: odra_core::consts::IS_UPGRADABLE_ARG.to_string(),
                description: None,
                ty: Type::System(CLType::Bool),
                optional: false
            },
        ]
        .iter()
        .chain(constructor_args.iter())
        .cloned()
        .collect()
    }
}
