use odra::contract_env;
use odra::types::casper_types::U512;

#[odra::module]
pub struct PublicWallet {}

#[odra::module]
impl PublicWallet {
    #[odra(payable)]
    pub fn deposit(&mut self) {}

    pub fn withdraw(&mut self, amount: &U512) {
        contract_env::transfer_tokens(&contract_env::caller(), *amount);
    }
}

#[cfg(test)]
mod tests {
    use super::PublicWalletDeployer;
    use odra::test_env;
    use odra::types::casper_types::U512;

    #[test]
    fn test_modules() {
        let mut my_contract = PublicWalletDeployer::default();
        assert_eq!(
            test_env::token_balance(*my_contract.address()),
            U512::zero()
        );

        my_contract.with_tokens(U512::from(100)).deposit();
        assert_eq!(
            test_env::token_balance(*my_contract.address()),
            U512::from(100)
        );

        my_contract.withdraw(&U512::from(25));
        assert_eq!(
            test_env::token_balance(*my_contract.address()),
            U512::from(75)
        );
    }
}
