use odra::{
    casper_types::{
        bytesrepr::{Bytes, ToBytes},
        Key, URef, U512
    },
    prelude::*
};

#[odra::module]
pub struct MainPurse {}

#[odra::module]
impl MainPurse {
    pub fn main_purse(&self) -> URef {
        self.__env.purse()
    }
}
