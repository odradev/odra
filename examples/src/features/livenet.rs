//! This is an example contract used to showcase and test Livenet Environment.
use crate::features::livenet::Error::{EmptyStack, SillyError};
use odra::casper_types::U256;
use odra::module::Revertible;
use odra::prelude::*;
use odra::ContractRef;
use odra_modules::access::Ownable;
use odra_modules::erc20::Erc20ContractRef;

/// Contract used by the Livenet examples.
#[odra::module(errors = Error)]
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
        let caller = self.env().caller();
        self.creator.set(caller);
        self.ownable.init(caller);
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
        self.stack.pop().unwrap_or_revert(self)
    }

    /// Returns the length of the stack.
    pub fn get_stack_len(&self) -> u32 {
        self.stack.len()
    }

    /// Returns the value on top of the stack, reverts if the stack is empty.
    pub fn peek(&self) -> u64 {
        match self.stack.len() {
            0 => self.revert(EmptyStack),
            len => self.stack.get(len - 1).unwrap_or_revert(self)
        }
    }

    /// Calls [peek](Self::peek) of this contract as another contract would.
    pub fn nested_peek(&self) -> u64 {
        LivenetContractContractRef::new(self.env(), self.env().self_address()).peek()
    }

    /// Returns the number of frames on the call stack.
    pub fn call_stack_depth(&self) -> u32 {
        self.env().call_stack().len() as u32
    }

    /// Returns the total supply of the ERC20 contract. This is an example of an immutable cross-contract call.
    pub fn immutable_cross_call(&self) -> U256 {
        Erc20ContractRef::new(
            self.env(),
            self.erc20_address.get().unwrap_or_revert(&self.env())
        )
        .total_supply()
    }

    /// Transfers 1 token from the ERC20 contract to the caller. This is an example of a mutable cross-contract call.
    pub fn mutable_cross_call(&mut self) {
        Erc20ContractRef::new(
            self.env(),
            self.erc20_address.get().unwrap_or_revert(&self.env())
        )
        .transfer(&self.env().caller(), &1.into());
    }

    /// Function that reverts with a silly error.
    pub fn function_that_reverts(&mut self) {
        self.revert(SillyError)
    }

    /// Function that eats up a lot of gas.
    pub fn expensive_function(&mut self) {
        let mut i = 0;
        while i < 50 {
            self.push_on_stack(i);
            self.pop_from_stack();
            i += 1;
        }
    }
}

/// Errors that can occur in the `LivenetContract` module.
#[odra::odra_error]
pub enum Error {
    /// Silly error.
    SillyError = 1,
    /// The stack is empty.
    EmptyStack = 2
}

#[cfg(test)]
mod tests {
    use crate::features::livenet::Error::EmptyStack;
    use crate::features::livenet::{LivenetContract, LivenetContractInitArgs};
    use alloc::string::ToString;
    use odra::host::Deployer;
    use odra::prelude::Addressable;
    use odra_modules::erc20::{Erc20, Erc20InitArgs};

    #[test]
    fn livenet_contract_test() {
        let test_env = odra_test::env();
        let mut erc20 = Erc20::deploy(
            &test_env,
            Erc20InitArgs {
                name: "TestToken".to_string(),
                symbol: "TT".to_string(),
                decimals: 18,
                initial_supply: Some(100_000.into())
            }
        );
        let mut livenet_contract = LivenetContract::deploy(
            &test_env,
            LivenetContractInitArgs {
                erc20_address: erc20.address()
            }
        );

        erc20.transfer(&livenet_contract.address(), &1000.into());

        livenet_contract.push_on_stack(1);
        assert_eq!(livenet_contract.pop_from_stack(), 1);
        livenet_contract.mutable_cross_call();
    }

    #[test]
    fn reverting_getter_returns_the_error() {
        let test_env = odra_test::env();
        let erc20 = Erc20::deploy(
            &test_env,
            Erc20InitArgs {
                name: "TestToken".to_string(),
                symbol: "TT".to_string(),
                decimals: 18,
                initial_supply: None
            }
        );
        let mut livenet_contract = LivenetContract::deploy(
            &test_env,
            LivenetContractInitArgs {
                erc20_address: erc20.address()
            }
        );

        let depth = livenet_contract.call_stack_depth();
        assert_eq!(livenet_contract.try_peek(), Err(EmptyStack.into()));
        assert_eq!(livenet_contract.try_nested_peek(), Err(EmptyStack.into()));
        assert_eq!(livenet_contract.call_stack_depth(), depth);

        livenet_contract.push_on_stack(7);
        assert_eq!(livenet_contract.peek(), 7);
        assert_eq!(livenet_contract.nested_peek(), 7);
    }
}
