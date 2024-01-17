use odra::prelude::*;
use odra::{Module, U512};

#[odra::module]
pub struct PublicWallet {}

#[odra::module]
impl PublicWallet {
    #[odra(payable)]
    pub fn deposit(&mut self) {}

    pub fn withdraw(&mut self, amount: &U512) {
        self.env().transfer_tokens(&self.env().caller(), amount);
    }
}

#[cfg(test)]
mod tests {
    use super::PublicWalletDeployer;
    use odra::U512;

    #[test]
    fn test_modules() {
        let test_env = odra_test::env();
        let mut my_contract = PublicWalletDeployer::init(&test_env);
        assert_eq!(test_env.balance_of(&my_contract.address()), U512::zero());

        my_contract.with_tokens(U512::from(100)).deposit();
        assert_eq!(test_env.balance_of(&my_contract.address()), U512::from(100));

        my_contract.withdraw(U512::from(25));
        assert_eq!(test_env.balance_of(&my_contract.address()), U512::from(75));
    }
}
