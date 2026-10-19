pub struct Token;

#[odra::external_contract]
impl Token {
    pub fn total_supply(&self) -> u64 {
        0
    }
}

fn main() {}
