//! This example shows how to test a contract.
use odra::prelude::*;

/// Contract presenting the testing abilities of the Odra Framework
#[odra::module]
pub struct TestingContract {
    name: Var<String>,
    created_at: Var<u64>,
    created_by: Var<Address>
}

/// Implementation of the TestingContract
#[odra::module]
impl TestingContract {
    /// Initializes the contract with the name
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.created_at.set(self.env().get_block_time());
        self.created_by.set(self.env().caller())
    }

    /// Returns the name of the contract
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Returns the creation time of the contract
    pub fn created_at(&self) -> u64 {
        self.created_at.get().unwrap()
    }

    /// Returns the address of the creator of the contract
    pub fn created_by(&self) -> Address {
        self.created_by.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use odra::{host::Deployer, prelude::*};

    use crate::features::testing::{TestingContract, TestingContractInitArgs};

    #[test]
    fn env() {
        let test_env = odra_test::env();
        test_env.set_caller(test_env.get_account(0));
        let testing_contract = TestingContract::deploy(
            &test_env,
            TestingContractInitArgs {
                name: "MyContract".to_string()
            }
        );
        let creator = testing_contract.created_by();
        test_env.set_caller(test_env.get_account(1));
        let testing_contract2 = TestingContract::deploy(
            &test_env,
            TestingContractInitArgs {
                name: "MyContract2".to_string()
            }
        );
        let creator2 = testing_contract2.created_by();
        assert_ne!(creator, creator2);
    }
}
