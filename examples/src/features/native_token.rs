use odra::casper_types::U512;
use odra::module::Module;
use odra::prelude::*;

#[odra::module]
pub struct PublicWallet;

#[odra::module]
impl PublicWallet {
    #[odra(payable)]
    pub fn deposit(&mut self) {}

    pub fn withdraw(&mut self, amount: &U512) {
        self.env().transfer_tokens(&self.env().caller(), amount);
    }

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
    fn test_modules() {
        let test_env = odra_test::env();
        let mut my_contract = PublicWalletHostRef::deploy(&test_env, NoArgs);
        let original_contract_balance = test_env.balance_of(my_contract.address());
        assert_eq!(test_env.balance_of(my_contract.address()), U512::zero());

        my_contract.with_tokens(U512::from(100)).deposit();
        assert_eq!(test_env.balance_of(my_contract.address()), U512::from(100));

        my_contract.withdraw(U512::from(25));
        assert_eq!(test_env.balance_of(my_contract.address()), U512::from(75));

        let contract_balance = my_contract.balance();
        assert_eq!(contract_balance, original_contract_balance + U512::from(75));
    }
}
