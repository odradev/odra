pub struct NativeTokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8
}

impl NativeTokenMetadata {
    pub(crate) fn new() -> Self {
        Self {
            name: String::from("Plascoin"),
            symbol: String::from("PLS"),
            decimals: 2
        }
    }
}
