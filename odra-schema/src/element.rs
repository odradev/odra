use casper_contract_schema::{CustomType, Type};
use odra_core::args::EntrypointArgument;

use crate::SchemaCustomType;

pub trait SchemaElement {
    fn ty() -> Type;
}

impl<T: SchemaCustomType + EntrypointArgument> SchemaElement for T {
    fn ty() -> Type {
        match T::custom_ty() {
            Some(CustomType::Enum { name, .. } | CustomType::Struct { name, .. }) => {
                Type::Custom(name)
            }
            None => T::ty().into()
        }
    }
}
