//! Better address representation for Casper.

use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Serialize, Deserialize)]
pub struct Address([u8; 20]);

impl Address {
    pub fn new(bytes: &[u8]) -> Address {
        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(20, 0);

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Address(bytes)
    }
}

impl Into<Addr> for Address {
    fn into(self) -> Addr {
        let str = String::from_utf8(self.0.to_vec()).unwrap();
        Addr::unchecked(str)
    }
}

impl Into<String> for Address {
    fn into(self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }
}

impl Into<String> for &Address {
    fn into(self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }
}
