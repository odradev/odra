use odra::contract_env;
use odra::types::Balance;

#[odra::module]
pub struct PublicWallet {}

#[odra::module]
impl PublicWallet {
    #[odra(payable)]
    pub fn deposit(&mut self) {}

    pub fn withdraw(&mut self, amount: &Balance) {
        contract_env::transfer_tokens(contract_env::caller(), *amount);
    }
}

#[cfg(test)]
mod tests {
    use super::PublicWalletDeployer;
    use odra::test_env;
    use odra::types::Balance;

    #[test]
    fn test_modules() {
        let mut my_contract = PublicWalletDeployer::default();
        assert_eq!(
            test_env::token_balance(my_contract.address()),
            Balance::zero()
        );

        my_contract.with_tokens(Balance::from(100)).deposit();
        assert_eq!(
            test_env::token_balance(my_contract.address()),
            Balance::from(100)
        );

        my_contract.withdraw(&Balance::from(25));
        assert_eq!(
            test_env::token_balance(my_contract.address()),
            Balance::from(75)
        );
    }
}
