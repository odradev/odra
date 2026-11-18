//! An external contract trait lists the entry points of one deployed contract.
use odra::prelude::*;

#[odra::external_contract(name = "Token")]
pub trait WithArgs {
    fn get(&self) -> u32;
}

#[odra::external_contract]
pub trait Generic<T> {
    fn get(&self) -> T;
}

#[odra::external_contract]
pub trait WithBody {
    fn get(&self) -> u32 {
        0
    }
}

#[odra::external_contract]
pub trait WithAssocItems {
    type Out;
    const LIMIT: u32;
    fn get(&self) -> u32;
}

#[odra::external_contract]
pub trait WithSupertrait: Clone {
    fn get(&self) -> u32;
}

#[odra::external_contract]
pub trait NoReceiver {
    fn get() -> u32;
}

#[odra::external_contract]
pub trait GenericFn {
    fn get<T>(&self) -> u32;
}

#[odra::external_contract]
pub trait Offchain {
    #[odra(offchain)]
    fn get(&self) -> u32;
}

#[odra::external_contract]
pub trait PubFn {
    pub fn get(&self) -> u32;
}

fn main() {}
