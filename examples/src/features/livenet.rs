use odra::casper_types::U256;
use odra::prelude::*;
use odra::{Address, List, SubModule, UnwrapOrRevert, Var};
use odra_modules::access::Ownable;
use odra_modules::erc20::Erc20ContractRef;

/// Contract used by the Livenet examples.
#[odra::module]
pub struct LivenetContract {
    creator: Var<Address>,
    ownable: SubModule<Ownable>,
    stack: List<u64>,
    erc20_address: Var<Address>
}

#[odra::module]
impl LivenetContract {
    /// Initializes the contract.
    pub fn init(mut self, erc20_address: Address) {
        self.creator.set(self.env().caller());
        self.ownable.init();
        self.erc20_address.set(erc20_address);
    }

    /// Transfers the ownership of the contract to a new owner.
    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.ownable.transfer_ownership(&new_owner);
    }

    /// Returns the owner of the contract.
    pub fn owner(&self) -> Address {
        self.ownable.get_owner()
    }

    /// Pushes a value on the stack.
    pub fn push_on_stack(&mut self, value: u64) {
        self.stack.push(value);
    }

    /// Pops a value from the stack.
    pub fn pop_from_stack(&mut self) -> u64 {
        self.stack.pop().unwrap_or_revert(&self.env())
    }

    /// Returns the length of the stack.
    pub fn get_stack_len(&self) -> u32 {
        self.stack.len()
    }

    /// Returns the total supply of the ERC20 contract. This is an example of an immutable cross-contract call.
    pub fn immutable_cross_call(&self) -> U256 {
        Erc20ContractRef::new(self.env(), self.erc20_address.get().unwrap()).total_supply()
    }

    /// Transfers 1 token from the ERC20 contract to the caller. This is an example of a mutable cross-contract call.
    pub fn mutable_cross_call(&mut self) {
        Erc20ContractRef::new(self.env(), self.erc20_address.get().unwrap())
            .transfer(self.env().caller(), 1.into());
    }
}
