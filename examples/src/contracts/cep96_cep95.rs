//! An example of a CEP-95 token with CEP-96 contract metadata.
use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::U256;
use odra::prelude::*;
use odra_modules::cep95::{CEP95Interface, Cep95};
use odra_modules::cep96::{Cep96, Cep96ContractMetadata};

/// CEP-95 token with CEP-96 contract metadata.
#[odra::module]
pub struct Cep96Cep95 {
    token: SubModule<Cep95>,
    metadata: SubModule<Cep96>
}

#[odra::module]
impl Cep96Cep95 {
    /// Initializes the contract with CEP-95 token params and CEP-96 metadata.
    pub fn init(
        &mut self,
        name: String,
        symbol: String,
        contract_name: Option<String>,
        contract_description: Option<String>,
        contract_icon_uri: Option<String>,
        contract_project_uri: Option<String>
    ) {
        self.token.init(name, symbol);
        self.metadata.init(
            contract_name,
            contract_description,
            contract_icon_uri,
            contract_project_uri
        );
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
        to self.metadata {
            fn contract_name(&self) -> Option<String>;
            fn contract_description(&self) -> Option<String>;
            fn contract_icon_uri(&self) -> Option<String>;
            fn contract_project_uri(&self) -> Option<String>;
        }
    }

    /// Mints a new token with the given ID and metadata to the specified address.
    pub fn mint(&mut self, to: Address, token_id: U256, metadata: Vec<(String, String)>) {
        self.token.raw_mint(to, token_id, metadata);
    }

    /// Burns the token with the given ID.
    pub fn burn(&mut self, token_id: U256) {
        let owner = self.token.owner_of(token_id);
        let caller = self.env().caller();
        if Some(caller) == owner {
            self.token.raw_burn(token_id);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Cep96Cep95;
    use crate::contracts::cep96_cep95::{Cep96Cep95HostRef, Cep96Cep95InitArgs};
    use odra::{host::Deployer, prelude::*};
    use odra_test;

    fn deploy_contract() -> Cep96Cep95HostRef {
        let env = odra_test::env();
        Cep96Cep95::deploy(
            &env,
            Cep96Cep95InitArgs {
                name: "TestToken".to_string(),
                symbol: "TST".to_string(),
                contract_name: Some("My NFT Collection".to_string()),
                contract_description: Some("A test NFT collection".to_string()),
                contract_icon_uri: Some("https://example.com/icon.png".to_string()),
                contract_project_uri: Some("https://example.com".to_string())
            }
        )
    }

    #[test]
    fn test_cep96_metadata() {
        let contract = deploy_contract();

        // Verify all CEP-96 metadata fields
        assert_eq!(
            contract.contract_name(),
            Some("My NFT Collection".to_string())
        );
        assert_eq!(
            contract.contract_description(),
            Some("A test NFT collection".to_string())
        );
        assert_eq!(
            contract.contract_icon_uri(),
            Some("https://example.com/icon.png".to_string())
        );
        assert_eq!(
            contract.contract_project_uri(),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_cep96_partial_metadata() {
        let env = odra_test::env();
        let contract = Cep96Cep95::deploy(
            &env,
            Cep96Cep95InitArgs {
                name: "TestToken".to_string(),
                symbol: "TST".to_string(),
                contract_name: Some("Partial Collection".to_string()),
                contract_description: None,
                contract_icon_uri: None,
                contract_project_uri: Some("https://example.com".to_string())
            }
        );

        // Only set fields should have values
        assert_eq!(
            contract.contract_name(),
            Some("Partial Collection".to_string())
        );
        assert_eq!(contract.contract_description(), None);
        assert_eq!(contract.contract_icon_uri(), None);
        assert_eq!(
            contract.contract_project_uri(),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_cep95_with_cep96() {
        let env = odra_test::env();
        let mut contract = deploy_contract();

        // CEP-95 functionality should work
        assert_eq!(contract.name(), "TestToken");
        assert_eq!(contract.symbol(), "TST");

        // Mint a token
        let owner = env.caller();
        let token_id = 1.into();
        contract.mint(
            owner,
            token_id,
            vec![("key".to_string(), "value".to_string())]
        );

        assert_eq!(contract.owner_of(token_id), Some(owner));
        assert_eq!(contract.balance_of(owner), 1.into());

        // CEP-96 metadata should still be accessible
        assert_eq!(
            contract.contract_name(),
            Some("My NFT Collection".to_string())
        );
    }
}
