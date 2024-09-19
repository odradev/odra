//! A module providing functionality for defining the Casper Contract Schema.
//!
//! It includes traits for defining entrypoints, events, custom types, and errors, as well as functions
//! for creating various schema elements such as arguments, entrypoints, struct members, enum variants, custom types,
//! events, and errors.
use std::{collections::BTreeSet, env, path::PathBuf};

pub use casper_contract_schema;
use casper_contract_schema::{
    Access, Argument, CallMethod, ContractSchema, CustomType, Entrypoint, EnumVariant, Event,
    NamedCLType, StructMember, UserError
};

use convert_case::{Boundary, Case, Casing};

use odra_core::args::EntrypointArgument;

const CCSV: u8 = 1;

mod custom_type;
mod ty;

pub use ty::NamedCLTyped;

/// Trait representing schema entrypoints.
pub trait SchemaEntrypoints {
    /// Returns a vector of [Entrypoint]s.
    fn schema_entrypoints() -> Vec<Entrypoint>;
}

/// Trait representing schema events.
pub trait SchemaEvents {
    /// Returns a vector of [Event]s.
    fn schema_events() -> Vec<Event> {
        vec![]
    }

    /// Returns a vector of [CustomType]s.
    ///
    /// This method is used to define custom types that are used in the events.
    /// An event itself is a [CustomType] and can have a custom type as its payload.
    fn custom_types() -> Vec<Option<CustomType>> {
        vec![]
    }
}

/// Trait for defining custom types in a schema.
pub trait SchemaCustomTypes {
    /// Returns a vector of optional [CustomType]s.
    fn schema_types() -> Vec<Option<CustomType>> {
        vec![]
    }
}

/// A trait for defining schema user errors.
pub trait SchemaErrors {
    /// Returns a vector of [UserError]s.
    fn schema_errors() -> Vec<UserError> {
        vec![]
    }
}

/// Represents a custom element in the schema.
pub trait SchemaCustomElement {}

impl<T: SchemaCustomElement> SchemaErrors for T {}
impl<T: SchemaCustomElement> SchemaEvents for T {}

/// Creates a new argument.
pub fn argument<T: NamedCLTyped + EntrypointArgument>(name: &str) -> Argument {
    if T::is_required() {
        Argument::new(name, "", <T as NamedCLTyped>::ty())
    } else {
        Argument::new_opt(name, "", <T as NamedCLTyped>::ty())
    }
}

/// Creates a new entrypoint.
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

/// Creates a new struct member.
pub fn struct_member<T: NamedCLTyped>(name: &str) -> StructMember {
    StructMember {
        name: name.to_string(),
        description: None,
        ty: T::ty().into()
    }
}

/// Creates a new enum variant.
pub fn enum_typed_variant<T: NamedCLTyped>(name: &str, discriminant: u16) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: T::ty().into()
    }
}

/// Creates a new enum variant of type [NamedCLType::Unit].
pub fn enum_variant(name: &str, discriminant: u16) -> EnumVariant {
    enum_typed_variant::<()>(name, discriminant)
}

/// Creates a new enum variant of type [NamedCLType::Custom].
pub fn enum_custom_type_variant(name: &str, discriminant: u16, custom_type: &str) -> EnumVariant {
    EnumVariant {
        name: name.to_string(),
        description: None,
        discriminant,
        ty: NamedCLType::Custom(custom_type.into()).into()
    }
}

/// Creates a new [CustomType] of type struct.
pub fn custom_struct(name: &str, members: Vec<StructMember>) -> CustomType {
    CustomType::Struct {
        name: name.into(),
        description: None,
        members
    }
}

/// Creates a new [CustomType] of type enum.
pub fn custom_enum(name: &str, variants: Vec<EnumVariant>) -> CustomType {
    CustomType::Enum {
        name: name.into(),
        description: None,
        variants
    }
}

/// Creates a new [Event].  
pub fn event(name: &str) -> Event {
    Event {
        name: name.into(),
        ty: name.into()
    }
}

/// Creates a new [UserError].
pub fn error(name: &str, description: &str, discriminant: u16) -> UserError {
    UserError {
        name: name.into(),
        description: Some(description.into()),
        discriminant
    }
}

/// Creates an instance of [ContractSchema].
///
/// A contract schema is a representation of a smart contract's schema. It includes information about
/// the contract's metadata, entrypoints, events, custom types, and errors.
pub fn schema<T: SchemaEntrypoints + SchemaEvents + SchemaCustomTypes + SchemaErrors>(
    module_name: &str,
    contract_name: &str,
    contract_version: &str,
    authors: Vec<String>,
    repository: &str,
    homepage: &str
) -> ContractSchema {
    let entry_points = T::schema_entrypoints();
    let events = T::schema_events();
    let errors = T::schema_errors();
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
        errors
    }
}

/// Finds the path to the schema file for the given contract name.
pub fn find_schema_file_path(
    contract_name: &str,
    root_path: PathBuf
) -> Result<PathBuf, &'static str> {
    let mut path = root_path
        .join(format!("{}_schema.json", camel_to_snake(contract_name)))
        .with_extension("json");

    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() && path.is_file() {
            return Ok(path);
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    Err("Schema not found")
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

/// Converts a string from camel case to snake case.
pub fn camel_to_snake<T: ToString>(text: T) -> String {
    text.to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

#[cfg(test)]
mod test {
    use odra_core::{args::Maybe, Address};

    use super::*;

    #[test]
    fn test_argument() {
        let arg = super::argument::<u32>("arg1");
        assert_eq!(arg.name, "arg1");
        assert_eq!(arg.ty, casper_contract_schema::NamedCLType::U32.into());
    }

    #[test]
    fn test_opt_argument() {
        let arg = super::argument::<Maybe<u32>>("arg1");
        assert_eq!(arg.name, "arg1");
        assert_eq!(arg.ty, casper_contract_schema::NamedCLType::U32.into());
    }

    #[test]
    fn test_entry_point() {
        let arg = super::argument::<u32>("arg1");
        let entry_point = super::entry_point::<u32>("entry1", "description", true, vec![arg]);
        assert_eq!(entry_point.name, "entry1");
        assert_eq!(entry_point.description, Some("description".to_string()));
        assert!(entry_point.is_mutable);
        assert_eq!(entry_point.arguments.len(), 1);
        assert_eq!(
            entry_point.return_ty,
            casper_contract_schema::NamedCLType::U32.into()
        );
    }

    #[test]
    fn test_struct_member() {
        let member = super::struct_member::<u32>("member1");
        assert_eq!(member.name, "member1");
        assert_eq!(member.ty, casper_contract_schema::NamedCLType::U32.into());
    }

    #[test]
    fn test_enum_typed_variant() {
        let variant = super::enum_typed_variant::<Address>("variant1", 1);
        assert_eq!(variant.name, "variant1");
        assert_eq!(variant.discriminant, 1);
        assert_eq!(variant.ty, casper_contract_schema::NamedCLType::Key.into());
    }

    #[test]
    fn test_enum_variant() {
        let variant = super::enum_variant("variant1", 1);
        assert_eq!(variant.name, "variant1");
        assert_eq!(variant.discriminant, 1);
        assert_eq!(variant.ty, casper_contract_schema::NamedCLType::Unit.into());
    }

    #[test]
    fn test_custom_struct() {
        let member = super::struct_member::<u32>("member1");
        let custom_struct = super::custom_struct("struct1", vec![member]);
        match custom_struct {
            casper_contract_schema::CustomType::Struct { name, members, .. } => {
                assert_eq!(name, "struct1".into());
                assert_eq!(members.len(), 1);
            }
            _ => panic!("Expected CustomType::Struct")
        }
    }

    #[test]
    fn test_custom_enum() {
        let variant1 = super::enum_variant("variant1", 1);
        let variant2 = super::enum_typed_variant::<String>("v2", 2);
        let variant3 = super::enum_custom_type_variant("v3", 3, "Type1");
        let custom_enum = super::custom_enum("enum1", vec![variant1, variant2, variant3]);
        match custom_enum {
            casper_contract_schema::CustomType::Enum { name, variants, .. } => {
                assert_eq!(name, "enum1".into());
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].ty, NamedCLType::Unit.into());
                assert_eq!(variants[1].ty, NamedCLType::String.into());
                assert_eq!(variants[2].ty, NamedCLType::Custom("Type1".into()).into());
            }
            _ => panic!("Expected CustomType::Enum")
        }
    }

    #[test]
    fn test_event() {
        let event = super::event("event1");
        assert_eq!(event.name, "event1");
    }

    #[test]
    fn test_error() {
        let error = super::error("error1", "description", 1);
        assert_eq!(error.name, "error1");
        assert_eq!(error.description, Some("description".to_string()));
        assert_eq!(error.discriminant, 1);
    }

    #[test]
    fn test_schema() {
        struct TestSchema;

        impl SchemaEntrypoints for TestSchema {
            fn schema_entrypoints() -> Vec<Entrypoint> {
                vec![entry_point::<u32>(
                    "entry1",
                    "description",
                    true,
                    vec![super::argument::<u32>("arg1")]
                )]
            }
        }

        impl SchemaEvents for TestSchema {
            fn schema_events() -> Vec<Event> {
                vec![event("event1")]
            }
        }

        impl SchemaCustomTypes for TestSchema {
            fn schema_types() -> Vec<Option<CustomType>> {
                vec![
                    Some(custom_struct(
                        "struct1",
                        vec![struct_member::<u32>("member1")]
                    )),
                    Some(custom_enum("struct1", vec![enum_variant("variant1", 1)])),
                ]
            }
        }

        impl SchemaErrors for TestSchema {
            fn schema_errors() -> Vec<UserError> {
                vec![]
            }
        }

        let schema = super::schema::<TestSchema>(
            "module_name",
            "contract_name",
            "contract_version",
            vec!["author".to_string()],
            "repository",
            "homepage"
        );

        assert_eq!(schema.contract_name, "contract_name");
        assert_eq!(schema.contract_version, "contract_version");
        assert_eq!(schema.authors, vec!["author".to_string()]);
        assert_eq!(schema.repository, Some("repository".to_string()));
        assert_eq!(schema.homepage, Some("homepage".to_string()));
        assert_eq!(schema.entry_points.len(), 1);
        assert_eq!(schema.types.len(), 2);
        assert_eq!(schema.errors.len(), 0);
        assert_eq!(schema.events.len(), 1);
    }
}
