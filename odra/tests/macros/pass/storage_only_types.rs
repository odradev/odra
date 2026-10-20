//! The contract schema describes the storage layout, and its `types` section includes the types
//! that are only stored and never passed to or returned from an entry point.
#![deny(warnings)]

use odra::prelude::*;
use odra::schema::casper_contract_schema::{ContractSchema, NamedCLType};
use odra::schema::{SchemaStorageLayout, StorageKind};

/// Stored in a `Var`.
#[odra::odra_type]
pub struct Stored {
    pub value: u32
}

/// A member of `Nested`, never stored directly.
#[odra::odra_type]
pub struct Inner {
    pub value: u32
}

/// Stored as a `Mapping` value.
#[odra::odra_type]
pub struct Nested {
    pub inner: Inner
}

/// Used as a `Mapping` key.
#[odra::odra_type]
pub struct MapKey {
    pub id: u32
}

/// Stored in a `List`.
#[odra::odra_type]
pub enum Listed {
    A,
    B
}

/// Stored by a submodule.
#[odra::odra_type]
pub struct InSubmodule {
    pub value: u32
}

#[odra::module]
pub struct Sub {
    value: Var<InSubmodule>
}

#[odra::module]
pub struct Contract {
    stored: Var<Stored>,
    nested: Mapping<MapKey, Nested>,
    listed: List<Listed>,
    sub: SubModule<Sub>,
    subs: Mapping<u32, Sub>,
    counter: Sequence<u32>
}

#[odra::module]
impl Contract {
    pub fn init(&mut self, value: u32) {
        self.stored.set(Stored { value });
    }

    pub fn value(&self) -> u32 {
        self.stored.get().map(|s| s.value).unwrap_or_default()
    }
}

fn schema() -> ContractSchema {
    odra::schema::schema::<Contract>("contract", "Contract", "1.0.0", vec![], "", "")
}

fn main() {
    let schema = schema();

    let mut names = schema.types.iter().map(|t| t.name()).collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        ["InSubmodule", "Inner", "Listed", "MapKey", "Nested", "Stored"]
    );

    assert_eq!(schema.storage, Contract::storage_kind());
    let fields = schema.storage.fields();
    let names = fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>();
    assert_eq!(
        names,
        ["stored", "nested", "listed", "sub", "subs", "counter"]
    );
    assert_eq!(
        fields[0].kind,
        StorageKind::value(NamedCLType::Custom("Stored".into()))
    );
    assert_eq!(
        fields[1].kind,
        StorageKind::mapping(
            NamedCLType::Custom("MapKey".into()),
            StorageKind::value(NamedCLType::Custom("Nested".into()))
        )
    );
    assert_eq!(
        fields[2].kind,
        StorageKind::list(NamedCLType::Custom("Listed".into()))
    );
    assert_eq!(fields[3].kind, Sub::storage_kind());
    assert_eq!(
        fields[4].kind,
        StorageKind::mapping(NamedCLType::U32, Sub::storage_kind())
    );
    assert_eq!(fields[5].kind, StorageKind::sequence(NamedCLType::U32));
}
