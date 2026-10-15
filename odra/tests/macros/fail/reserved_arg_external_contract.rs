#[odra::external_contract]
pub trait Token {
    fn transfer(&mut self, amount: u64, gas: u64);
}

fn main() {}
