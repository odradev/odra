pub struct NativeToken {
    name: String,
    symbol: String,
    decimals: u8,
}

impl NativeToken {
    pub fn new(name: &str, symbol: &str, decimals: u8) -> Self {
        Self { name: String::from(name), symbol: String::from(symbol), decimals }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }
}
