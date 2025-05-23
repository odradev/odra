//! An example of a OwnedCep95 contract.
use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::U256;
use odra::prelude::*;
use odra_modules::access::Ownable;
use odra_modules::cep95::{CEP95Interface, Cep95};

/// OwnedCep95 contract.
#[odra::module]
pub struct OwnedCep95 {
    ownable: SubModule<Ownable>,
    token: SubModule<Cep95>
}

#[odra::module]
impl OwnedCep95 {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, name: String, symbol: String) {
        let owner = self.env().caller();
        self.ownable.init(owner);

        self.token.name.set(name);
        self.token.symbol.set(symbol);
    }

    delegate! {
        to self.token {
            fn name(&self) -> String;
            fn symbol(&self) -> String;
            fn balance_of(&self, owner: Address) -> U256;
            fn owner_of(&self, token_id: U256) -> Option<Address>;
            fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256, data: Option<Bytes>);
            fn transfer_from(&mut self, from: Address, to: Address, token_id: U256);
            fn approve(&mut self, spender: Address, token_id: U256);
            fn revoke_approval(&mut self, token_id: U256);
            fn approved_for(&self, token_id: U256) -> Option<Address>;
            fn approve_for_all(&mut self, operator: Address);
            fn revoke_approval_for_all(&mut self, operator: Address);
            fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;
            fn token_metadata(&self, token_id: U256) -> Vec<(String, String)>;
        }
    }

    delegate! {
        to self.ownable {
            fn get_owner(&self) -> Address;
            fn transfer_ownership(&mut self, new_owner: &Address);
        }
    }

    /// Mints a new token with the given ID and metadata to the specified address.
    /// Only the contract owner can call this function.
    pub fn mint(&mut self, to: Address, token_id: U256, metadata: Vec<(String, String)>) {
        self.ownable.assert_owner(&self.env().caller());
        self.token.mint(to, token_id, metadata);
    }

    /// Burns the token with the given ID.
    /// Only the token owner can call this function.
    /// This function will remove the token from the owner's balance and delete its metadata.
    pub fn burn(&mut self, token_id: U256) {
        let owner = self.token.owner_of(token_id);
        let caller = self.env().caller();
        if Some(caller) == owner {
            self.token.burn(token_id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::OwnedCep95;
    use crate::contracts::owned_cep95::OwnedCep95InitArgs;
    use odra::{host::Deployer, prelude::*};
    use odra_test;

    #[test]
    fn test_init() {
        let env = odra_test::env();
        let contract = OwnedCep95::deploy(
            &env,
            OwnedCep95InitArgs {
                name: "Test".to_string(),
                symbol: "TST".to_string()
            }
        );
        assert_eq!(contract.name(), "Test");
        assert_eq!(contract.symbol(), "TST");
    }

    #[test]
    fn test_mint() {
        let env = odra_test::env();
        let mut contract = OwnedCep95::deploy(
            &env,
            OwnedCep95InitArgs {
                name: "Test".to_string(),
                symbol: "TST".to_string()
            }
        );
        let owner = env.caller();
        let token_id = 1.into();

        contract.mint(
            owner,
            token_id,
            vec![("key".to_string(), "value".to_string())]
        );
        assert_eq!(contract.owner_of(token_id), Some(owner));
    }

    #[test]
    fn test_burn() {
        let env = odra_test::env();
        let mut contract = OwnedCep95::deploy(
            &env,
            OwnedCep95InitArgs {
                name: "Test".to_string(),
                symbol: "TST".to_string()
            }
        );
        let owner = env.caller();
        let token_id = 1.into();

        contract.mint(
            owner,
            token_id,
            vec![("key".to_string(), "value".to_string())]
        );
        contract.burn(token_id);
        assert_eq!(contract.owner_of(token_id), None);
    }
}
