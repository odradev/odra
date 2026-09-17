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
        self.env()
            .debug(format!("TestingContract::init called with name {name:?}"));
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
        self.created_at.get().unwrap_or_revert(&self.env())
    }

    /// Returns the address of the creator of the contract
    pub fn created_by(&self) -> Address {
        self.created_by.get().unwrap_or_revert(&self.env())
    }
}

#[cfg(test)]
mod tests {
    use crate::contracts::owned_token::{OwnedToken, OwnedTokenInitArgs};
    use crate::features::testing::{TestingContract, TestingContractInitArgs};
    use core::time::Duration;
    use odra::casper_types::{U256, U512};
    use odra::{
        host::{Deployer, HostEnv, HostRefLoader},
        prelude::*
    };
    use odra_modules::access::{Ownable, OwnableInitArgs};

    #[test]
    fn env() {
        let test_env: HostEnv = odra_test::env();
        test_env.set_caller(test_env.get_account(0));
        let init_args = TestingContractInitArgs {
            name: "MyContract".to_string()
        };
        let testing_contract = TestingContract::deploy(&test_env, init_args);
        let creator = testing_contract.created_by();
        test_env.set_caller(test_env.get_account(1));
        let init_args = TestingContractInitArgs {
            name: "MyContract2".to_string()
        };
        let testing_contract2 = TestingContract::deploy(&test_env, init_args);
        let creator2 = testing_contract2.created_by();
        assert_ne!(creator, creator2);
    }

    #[test]
    fn snapshot_and_restore() {
        let env = odra_test::env();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let mut token = OwnedToken::deploy(
            &env,
            OwnedTokenInitArgs {
                name: "Token".to_string(),
                symbol: "TKN".to_string(),
                decimals: 3,
                initial_supply: U256::from(1_000)
            }
        );

        // Everything below branches off this point.
        env.take_snapshot();
        let block_time = env.block_time();
        let events_count = env.events_count(&token);
        let alice_cspr = env.balance_of(&alice);

        // Scenario A: a transfer, some CSPR and an hour pass.
        token.transfer(&alice, &U256::from(100));
        env.transfer(alice, U512::from(1_000_000_000u64)).unwrap();
        env.advance_block_time(Duration::from_secs(60 * 60));
        assert_eq!(token.balance_of(&alice), U256::from(100));
        assert_eq!(env.events_count(&token), events_count + 1);
        assert_ne!(env.block_time(), block_time);
        assert_ne!(env.balance_of(&alice), alice_cspr);

        // Back to the starting point: scenario A never happened.
        env.restore_snapshot();
        assert_eq!(token.balance_of(&alice), U256::zero());
        assert_eq!(token.balance_of(&owner), U256::from(1_000));
        assert_eq!(env.events_count(&token), events_count);
        assert_eq!(env.block_time(), block_time);
        assert_eq!(env.balance_of(&alice), alice_cspr);

        // Scenario B starts from the same point, and the snapshot can be restored again.
        token.transfer(&alice, &U256::from(1));
        assert_eq!(token.balance_of(&alice), U256::from(1));
        env.restore_snapshot();
        assert_eq!(token.balance_of(&alice), U256::zero());
    }

    #[test]
    fn concurrently_deploys_and_reads() {
        // On the VMs `concurrently` runs the items one after another; on livenet they are spread
        // over worker threads. The code is the same either way.
        let env = odra_test::env();
        let supplies = [U256::from(10), U256::from(20), U256::from(30)];

        let addresses = env.concurrently(supplies.to_vec(), |env, initial_supply| {
            OwnedToken::deploy(
                env,
                OwnedTokenInitArgs {
                    name: "Token".to_string(),
                    symbol: "TKN".to_string(),
                    decimals: 0,
                    initial_supply
                }
            )
            .address()
        });

        let read = env.concurrently(addresses, |env, address| {
            OwnedToken::load(env, address).total_supply()
        });
        assert_eq!(read, supplies.to_vec());
    }

    #[test]
    fn odra_vm_only() {
        // `odra_test::odra_env()` ignores `ODRA_BACKEND`, so this test runs on OdraVM
        // even under `cargo odra test -b casper`. Handy for a submodule that is not
        // registered as a contract in `Odra.toml` and has no wasm built.
        let test_env = odra_test::odra_env();
        let owner = test_env.get_account(0);
        let ownable = Ownable::deploy(&test_env, OwnableInitArgs { owner });
        assert_eq!(ownable.get_owner(), owner);
    }
}
