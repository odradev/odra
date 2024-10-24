//! This example demonstrates how to handle errors in a contract.
use odra::prelude::*;

/// Contract that has an owner.
#[odra::module(errors = Error)]
pub struct OwnedContract {
    name: Var<String>,
    owner: Var<Address>
}

/// Errors that can occur in the `OwnedContract` module.
#[odra::odra_error]
pub enum Error {
    /// The owner is not set.
    OwnerNotSet = 1,
    /// The caller is not the owner.
    NotAnOwner = 2
}

#[odra::module]
impl OwnedContract {
    /// Initializes the contract with the given name.
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.owner.set(self.env().caller())
    }

    /// Returns the contract's name.
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Returns the contract's owner.
    pub fn owner(&self) -> Address {
        self.owner.get_or_revert_with(Error::OwnerNotSet)
    }

    /// Changes the contract's name.
    pub fn change_name(&mut self, name: String) {
        let caller = self.env().caller();
        if caller != self.owner() {
            self.env().revert(Error::NotAnOwner)
        }

        self.name.set(name);
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, OwnedContract, OwnedContractInitArgs};
    use odra::{host::Deployer, prelude::*};

    #[test]
    fn test_owner() {
        let test_env = odra_test::env();
        let owner = test_env.get_account(0);
        let not_an_owner = test_env.get_account(1);

        test_env.set_caller(owner);
        let mut owned_contract = OwnedContract::deploy(
            &test_env,
            OwnedContractInitArgs {
                name: "OwnedContract".to_string()
            }
        );

        test_env.set_caller(not_an_owner);
        assert_eq!(
            owned_contract
                .try_change_name("NewName".to_string())
                .unwrap_err(),
            Error::NotAnOwner.into()
        );
        assert_ne!(owned_contract.name(), "NewName");
    }

    #[test]
    fn test_owner_error() {
        let test_env = odra_test::env();
        let owner = test_env.get_account(0);
        let not_an_owner = test_env.get_account(1);

        test_env.set_caller(owner);
        let mut owned_contract = OwnedContract::deploy(
            &test_env,
            OwnedContractInitArgs {
                name: "OwnedContract".to_string()
            }
        );

        test_env.set_caller(not_an_owner);
        assert_eq!(
            owned_contract
                .try_change_name("NewName".to_string())
                .unwrap_err(),
            Error::NotAnOwner.into()
        );
    }
}
