use std::{collections::BTreeSet, ops::Deref};

use odra::schema::{casper_contract_schema::CustomType, SchemaCustomTypes, SchemaEvents};

pub(crate) type CustomTypeSet = BTreeSet<CustomType>;

#[derive(Debug, Default)]
/// CustomTypes is a struct that holds a set of custom types used in the Odra CLI.
pub struct CustomTypes {
    set: CustomTypeSet
}

impl CustomTypes {
    /// Returns a list of custom types used in the Odra CLI.
    pub fn register<T: SchemaCustomTypes + SchemaEvents>(&mut self) {
        self.set.extend(T::schema_types().into_iter().flatten());
        self.set
            .extend(<T as SchemaEvents>::custom_types().into_iter().flatten());
    }
}

impl Deref for CustomTypes {
    type Target = CustomTypeSet;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}
