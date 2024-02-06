use odra::prelude::*;
use odra::{Address, Var};

#[odra::module]
pub struct TestingContract {
    name: Var<String>,
    created_at: Var<u64>,
    created_by: Var<Address>
}

#[odra::module]
impl TestingContract {
    pub fn init(&mut self, name: String) {
        self.name.set(name);
        self.created_at.set(self.env().get_block_time());
        self.created_by.set(self.env().caller())
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn created_at(&self) -> u64 {
        self.created_at.get().unwrap()
    }

    pub fn created_by(&self) -> Address {
        self.created_by.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use odra::{host::Deployer, prelude::*};

    use crate::features::testing::{TestingContractHostRef, TestingContractInitArgs};

    #[test]
    fn env() {
        let test_env = odra_test::env();
        test_env.set_caller(test_env.get_account(0));
        let testing_contract = TestingContractHostRef::deploy(
            &test_env,
            TestingContractInitArgs {
                name: "MyContract".to_string()
            }
        );
        let creator = testing_contract.created_by();
        test_env.set_caller(test_env.get_account(1));
        let testing_contract2 = TestingContractHostRef::deploy(
            &test_env,
            TestingContractInitArgs {
                name: "MyContract2".to_string()
            }
        );
        let creator2 = testing_contract2.created_by();
        assert_ne!(creator, creator2);
    }
}
