//! Entry points named like a function the host ref generates for another one: `try_x` for
//! every entry point `x`, `x_no_ret` and `try_x_no_ret` for one taking `&mut self` and
//! returning a value. Reported on the function, not as a duplicate definition on the macro.
use odra::prelude::*;

#[odra::module]
pub struct Token {
    value: Var<u32>
}

#[odra::module]
impl Token {
    pub fn transfer(&mut self, amount: u32) {
        self.value.set(amount);
    }

    pub fn try_transfer(&mut self, amount: u32) {
        self.value.set(amount);
    }

    pub fn swap(&mut self, amount: u32) -> u32 {
        amount
    }

    pub fn swap_no_ret(&mut self, amount: u32) {
        self.value.set(amount);
    }

    // `try_burn_no_ret` is the `_no_ret` version of `try_burn` and the `try_` version of
    // `burn_no_ret`.
    pub fn try_burn(&mut self) -> u32 {
        0
    }

    pub fn burn_no_ret(&self) {}
}

#[odra::external_contract]
pub trait External {
    fn try_approve(&mut self, amount: u32);
    fn approve(&mut self, amount: u32);
}

fn main() {}
