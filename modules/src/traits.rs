use odra::types::{Balance, Address};

pub trait Mintable {
    fn mint(&self, address: Address, amount: Balance);
}

pub trait Burnable {
    fn burn(&self, address: Address, amount: Balance);
}