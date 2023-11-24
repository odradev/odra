use crate::prelude::string::String;

pub struct NativeTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8
}

impl Default for NativeTokenMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeTokenMetadata {
    pub fn new() -> Self {
        Self {
            name: String::from("Casper"),
            symbol: String::from("CSPR"),
            decimals: 9
        }
    }
}
