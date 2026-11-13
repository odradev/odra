//! The errors of all the modules of a contract share one range of codes; generating the schema
//! of a contract in which two of them use the same code fails, naming both.
#![cfg(not(target_arch = "wasm32"))]
#![allow(missing_docs, dead_code)]

use odra::prelude::*;

#[odra::odra_error]
pub enum OwnerError {
    /// The caller is not the owner.
    NotAnOwner = 1
}

#[odra::odra_error]
pub enum TokenError {
    /// Not enough tokens.
    InsufficientBalance = 1,
    /// The transfer is paused.
    Paused = 2
}

#[odra::module(errors = OwnerError)]
pub struct Owned {
    owner: Var<Address>
}

#[odra::module]
impl Owned {
    pub fn owner(&self) -> Address {
        self.owner.get_or_revert_with(OwnerError::NotAnOwner)
    }
}

#[odra::module(errors = TokenError)]
pub struct Token {
    supply: Var<u32>
}

#[odra::module]
impl Token {
    pub fn supply(&self) -> u32 {
        self.supply.get_or_revert_with(TokenError::Paused)
    }
}

/// Reaches `OwnerError` three times: its own, and through two `Owned` submodules.
#[odra::module(errors = OwnerError)]
pub struct TwiceOwned {
    first: SubModule<Owned>,
    second: SubModule<Owned>
}

#[odra::module]
impl TwiceOwned {
    pub fn owner(&self) -> Address {
        self.first.owner()
    }
}

/// `OwnerError::NotAnOwner` and `TokenError::InsufficientBalance` are both 1.
#[odra::module]
pub struct OwnedToken {
    owned: SubModule<Owned>,
    token: SubModule<Token>
}

#[odra::module]
impl OwnedToken {
    pub fn supply(&self) -> u32 {
        self.token.supply()
    }
}

fn schema_of<T>(name: &str) -> odra::schema::casper_contract_schema::ContractSchema
where
    T: odra::schema::SchemaEntrypoints
        + odra::schema::SchemaEvents
        + odra::schema::SchemaCustomTypes
        + odra::schema::SchemaErrors
        + odra::schema::SchemaStorageLayout
{
    odra::schema::schema::<T>(name, name, "1.0.0", vec![], "", "")
}

#[test]
fn an_error_enum_shared_by_modules_is_one_set_of_codes() {
    let schema = schema_of::<TwiceOwned>("TwiceOwned");
    assert_eq!(schema.errors.len(), 1);
    assert_eq!(schema_of::<Token>("Token").errors.len(), 2);
}

#[test]
#[should_panic(
    expected = "Contract `OwnedToken`: error code 1 is shared by `InsufficientBalance` and `NotAnOwner`"
)]
fn modules_with_the_same_error_code_cannot_make_one_contract() {
    schema_of::<OwnedToken>("OwnedToken");
}
