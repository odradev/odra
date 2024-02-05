use odra::casper_types::U256;
use odra::prelude::*;
use odra::{
    module::{Module, SubModule},
    Address, List, UnwrapOrRevert, Variable
};
use odra_modules::access::Ownable;
use odra_modules::erc20::Erc20ContractRef;

#[odra::module]
pub struct LivenetContract {
    creator: Variable<Address>,
    ownable: SubModule<Ownable>,
    stack: List<u64>,
    erc20_address: Variable<Address>
}

#[odra::module]
impl LivenetContract {
    pub fn init(mut self, erc20_address: Address) {
        self.creator.set(self.env().caller());
        self.ownable.init();
        self.erc20_address.set(erc20_address);
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

    pub fn immutable_cross_call(&self) -> U256 {
        Erc20ContractRef::new(self.env(), self.erc20_address.get().unwrap()).total_supply()
    }

    pub fn mutable_cross_call(&mut self) {
        Erc20ContractRef::new(self.env(), self.erc20_address.get().unwrap())
            .transfer(self.env().caller(), 1.into());
    }
}
