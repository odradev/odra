//! This example demonstrates how to handle native CSPR transfers in a contract.
use odra::casper_types::U512;
use odra::prelude::*;

/// Public wallet contract - used to show how Odra handles native CSPR transfers.
#[odra::module]
pub struct PublicWallet;

#[odra::module]
impl PublicWallet {
    /// Deposits the tokens into the contract.
    #[odra(payable)]
    pub fn deposit(&mut self) {}

    /// Withdraws the tokens from the contract.
    pub fn withdraw(&mut self, amount: &U512) {
        self.env().transfer_tokens(&self.env().caller(), amount);
    }

    /// Returns the balance of the contract.
    pub fn balance(&self) -> U512 {
        self.env().self_balance()
    }
}

#[cfg(test)]
mod tests {
    use super::PublicWalletHostRef;
    use odra::{
        casper_types::U512,
        host::{Deployer, HostRef, NoArgs}
    };

    #[test]
    fn test_public_wallet() {
        let test_env = odra_test::env();
        let mut my_contract = PublicWalletHostRef::deploy(&test_env, NoArgs);
        let original_contract_balance = test_env.balance_of(&my_contract);
        assert_eq!(test_env.balance_of(&my_contract), U512::zero());

        my_contract.with_tokens(U512::from(100)).deposit();
        assert_eq!(test_env.balance_of(&my_contract), U512::from(100));

        my_contract.withdraw(&U512::from(25));
        assert_eq!(test_env.balance_of(&my_contract), U512::from(75));

        let contract_balance = my_contract.balance();
        assert_eq!(contract_balance, original_contract_balance + U512::from(75));
    }

    #[test]
    fn test_call_non_payable_function_with_tokens() {
        let test_env = odra_test::env();
        let contract = PublicWalletHostRef::deploy(&test_env, NoArgs);
        let caller_address = test_env.get_account(0);
        let original_caller_balance = test_env.balance_of(&caller_address);

        contract.with_tokens(U512::from(100)).deposit();
        // call a non-payable function with tokens should fail and tokens should be refunded
        assert!(contract
            .with_tokens(U512::from(10))
            .try_withdraw(&U512::from(25))
            .is_err());
        // only the `deposit` function should have an effect
        assert_eq!(
            test_env.balance_of(&caller_address),
            original_caller_balance - U512::from(100)
        );
    }
}
