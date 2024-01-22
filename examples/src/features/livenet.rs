use odra::{Address, List, Module, ModuleWrapper, UnwrapOrRevert, Variable};
use odra::prelude::*;
use odra_modules::access::Ownable;

#[odra::module]
pub struct LivenetContract {
    creator: Variable<Address>,
    ownable: ModuleWrapper<Ownable>,
    stack: List<u64>,
}

#[odra::module]
impl LivenetContract {
    pub fn init(mut self) {
        self.creator.set(self.env().caller());
        self.ownable.init();
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.ownable.transfer_ownership(&new_owner);
    }

    pub fn owner(&self) -> Address {
        self.ownable.get_owner()
    }

    pub fn push_on_stack(&mut self, value: u64) {
        self.stack.push(value);
    }

    pub fn pop_from_stack(&mut self) -> u64 {
        self.stack.pop().unwrap_or_revert(&self.env())
    }

    pub fn get_stack_len(&self) -> u32 {
        self.stack.len()
    }
}