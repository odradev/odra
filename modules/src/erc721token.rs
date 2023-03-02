use crate::erc721::events::Transfer;
use odra::contract_env::{caller, revert};
use odra::types::event::OdraEvent;
use odra::types::Address;
use odra::types::U256;

use crate::erc721::erc721_base::Erc721Base;
use crate::erc721::errors::Error::{NotAnOwnerOrApproved, ZeroAddress};
use crate::erc721::extensions::erc721_metadata::{Erc721Metadata, Erc721MetadataExtension};
use crate::erc721::Erc721;
use crate::erc721token::errors::Error;
use crate::extensions::ownable::{Ownable, OwnableExtension};

#[odra::module]
pub struct Erc721Token {
    core: Erc721Base,
    metadata: Erc721MetadataExtension,
    ownable: OwnableExtension
}

#[odra::module]
impl OwnedErc721WithMetadata for Erc721Token {
    #[odra(init)]
    pub fn init(&mut self, name: String, symbol: String, base_uri: String) {
        self.metadata.init(name, symbol, base_uri);
        self.ownable.init(Some(caller()));
    }

    pub fn name(&self) -> String {
        self.metadata.name()
    }

    pub fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    pub fn base_uri(&self) -> String {
        self.metadata.base_uri()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.core.balance_of(owner)
    }

    pub fn owner_of(&self, token_id: U256) -> Address {
        self.core.owner_of(token_id)
    }

    pub fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        self.core.safe_transfer_from(from, to, token_id);
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        self.core.transfer_from(from, to, token_id);
    }

    pub fn approve(&mut self, approved: Address, token_id: U256) {
        self.core.approve(approved, token_id);
    }

    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) {
        self.core.set_approval_for_all(operator, approved);
    }

    pub fn get_approved(&self, token_id: U256) -> Option<Address> {
        self.core.get_approved(token_id)
    }

    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.core.is_approved_for_all(owner, operator)
    }

    pub fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership();
    }

    pub fn transfer_ownership(&mut self, new_owner: Option<Address>) {
        self.ownable.transfer_ownership(new_owner);
    }

    pub fn owner(&self) -> Option<Address> {
        self.ownable.owner()
    }

    pub fn mint(&mut self, to: Address, token_id: U256) {
        self.ownable.assert_owner(caller());

        if self.core.exists(&token_id) {
            revert(Error::TokenAlreadyExists)
        }

        if to.is_zero() {
            revert(ZeroAddress)
        }

        self.core.balances.add(&to, U256::from(1));
        self.core.owners.set(&token_id, Some(to));
    }

    pub fn burn(&mut self, token_id: U256) {
        self.core.assert_exists(&token_id);
        if !self.core.is_approved_or_owner(caller(), token_id) {
            revert(NotAnOwnerOrApproved);
        }

        let owner = self.core.owner_of(token_id);
        self.core
            .balances
            .set(&owner, self.core.balance_of(owner) - U256::from(1));
        self.core.owners.set(&token_id, None);
        self.core.clear_approval(token_id);

        Transfer {
            from: Some(owner),
            to: None,
            token_id
        }
        .emit();
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            TokenAlreadyExists => 35_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Erc721TokenDeployer, Erc721TokenRef};
    use crate::erc721::errors::Error;
    use crate::extensions::ownable::errors::Error::NotAnOwner;
    use odra::test_env;
    use odra::test_env::assert_exception;
    use odra::types::{Address, U256};

    const NAME: &str = "PlascoinNFT";
    const SYMBOL: &str = "PLSNFT";
    const BASE_URI: &str = "https://plascoin.org/";

    struct TokenEnv {
        token: Erc721TokenRef,
        alice: Address,
        bob: Address,
        carol: Address,
        zero: Address
    }

    fn setup() -> TokenEnv {
        TokenEnv {
            token: Erc721TokenDeployer::init(
                NAME.to_string(),
                SYMBOL.to_string(),
                BASE_URI.to_string()
            ),
            alice: test_env::get_account(1),
            bob: test_env::get_account(2),
            carol: test_env::get_account(3),
            zero: test_env::zero_address()
        }
    }

    #[test]
    fn mints_nft() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then Alice has a balance of 1 and Bob has a balance of 0.
        assert_eq!(erc721_env.token.balance_of(erc721_env.alice), U256::from(1));
        assert_eq!(erc721_env.token.balance_of(erc721_env.bob), U256::from(0));

        // And the owner of the token is Alice.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.alice);
    }

    #[test]
    fn balance_of() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then Alice has a balance of 1 and Bob has a balance of 0.
        assert_eq!(erc721_env.token.balance_of(erc721_env.alice), U256::from(1));

        // When we mint another token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(2));

        // Then Alice has a balance of 2.
        assert_eq!(erc721_env.token.balance_of(erc721_env.alice), U256::from(2));
    }

    #[test]
    fn zero_address() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then checking balance of zero address throws an error.
        assert_exception(Error::ZeroAddress, || {
            erc721_env.token.balance_of(erc721_env.zero);
        });
    }

    #[test]
    fn minting_same_id() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then minting the same token again throws an error.
        assert_exception(super::Error::TokenAlreadyExists, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.mint(erc721_env.alice, U256::from(1));
        });
    }

    #[test]
    fn minting_for_zero() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then minting a token to zero address throws an error.
        assert_exception(Error::ZeroAddress, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.mint(erc721_env.zero, U256::from(1));
        });
    }

    #[test]
    fn finding_owner() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then the owner of the token is Alice.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.alice);
    }

    #[test]
    fn finding_owner_of_non_existing_token() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then the owner of the token is Alice.
        assert_exception(Error::InvalidTokenId, || {
            erc721_env.token.owner_of(U256::from(1));
        });
    }

    #[test]
    fn approve() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And approve Bob to transfer the token.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.approve(erc721_env.bob, U256::from(1));

        // Then Bob is approved to transfer the token.
        assert_eq!(
            erc721_env.token.get_approved(U256::from(1)),
            Some(erc721_env.bob)
        );
    }

    #[test]
    fn cancel_approve() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And approve Bob to transfer the token.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.approve(erc721_env.bob, U256::from(1));

        // And cancel the approval.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.approve(erc721_env.zero, U256::from(1));

        // Then Bob is not approved to transfer the token.
        assert_eq!(erc721_env.token.get_approved(U256::from(1)), None);
    }

    #[test]
    fn approve_non_existing_token() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then approving a non existing token throws an error.
        assert_exception(Error::InvalidTokenId, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.approve(erc721_env.bob, U256::from(1));
        });
    }

    #[test]
    fn approve_not_owned_token() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then approving a token that is not owned by the caller throws an error.
        assert_exception(Error::NotAnOwnerOrApproved, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.approve(erc721_env.bob, U256::from(1));
        });
    }

    #[test]
    fn set_an_operator() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And set Bob as an operator.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, true);

        // Then Bob is an operator.
        assert!(erc721_env
            .token
            .is_approved_for_all(erc721_env.alice, erc721_env.bob));
    }

    #[test]
    fn cancel_an_operator() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And set Bob as an operator.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, true);

        // And cancel Bob as an operator.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, false);

        // Then Bob is not an operator.
        assert!(!erc721_env
            .token
            .is_approved_for_all(erc721_env.alice, erc721_env.bob));
    }

    #[test]
    fn transfer_nft() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And transfer the token to Bob.
        test_env::set_caller(erc721_env.alice);
        erc721_env
            .token
            .transfer_from(erc721_env.alice, erc721_env.bob, U256::from(1));

        // Then the owner of the token is Bob.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.bob);
    }

    #[test]
    fn transfer_approved() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And approve Bob to transfer the token.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.approve(erc721_env.bob, U256::from(1));

        // And transfer the token to Carol.
        test_env::set_caller(erc721_env.bob);
        erc721_env
            .token
            .transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1));

        // Then the owner of the token is Carol.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.carol);
    }

    #[test]
    fn transfer_operator() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And set Bob as an operator.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, true);

        // And transfer the token to Carol.
        test_env::set_caller(erc721_env.bob);
        erc721_env
            .token
            .transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1));

        // Then the owner of the token is Carol.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.carol);
    }

    #[test]
    fn transfer_not_owned() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then transferring a token that is not owned by the caller throws an error.
        assert_exception(Error::NotAnOwnerOrApproved, || {
            test_env::set_caller(erc721_env.bob);
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1));
        });
    }

    #[test]
    fn transferring_to_zero_address() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then transferring a token to the zero address throws an error.
        assert_exception(Error::ZeroAddress, || {
            test_env::set_caller(erc721_env.alice);
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.transfer_from(erc721_env.alice, erc721_env.zero, U256::from(1));
        });
    }

    #[test]
    fn transferring_invalid_nft() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then transferring a token that does not exist throws an error.
        assert_exception(Error::InvalidTokenId, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1));
        });
    }

    #[test]
    fn safe_transfer() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And safe transfer the token to Bob.
        test_env::set_caller(erc721_env.alice);
        erc721_env
            .token
            .safe_transfer_from(erc721_env.alice, erc721_env.bob, U256::from(1));

        // Then the owner of the token is Bob.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.bob);
    }

    #[test]
    fn safe_transfer_to_contract_which_does_not_support_nft() {
        todo!()
    }

    #[test]
    fn safe_transfer_to_contract_which_supports_nft() {
        todo!()
    }

    #[test]
    fn safe_transfer_to_contract_with_data() {
        todo!()
    }

    #[test]
    fn burn() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And burn the token.
        test_env::set_caller(erc721_env.alice);
        erc721_env.token.burn(U256::from(1));

        // Then the owner of throws an error.
        assert_exception(Error::InvalidTokenId, || {
            erc721_env.token.owner_of(U256::from(1));
        });
    }

    #[test]
    fn burn_non_existing_nft() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then burning a token that does not exist throws an error.
        assert_exception(Error::InvalidTokenId, || {
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.burn(U256::from(1));
        });
    }

    #[test]
    fn metadata() {
        // When deploy a contract with the initial supply.
        let erc721 = setup();

        // Then the contract has the metadata set.
        assert_eq!(erc721.token.symbol(), SYMBOL.to_string());
        assert_eq!(erc721.token.name(), NAME.to_string());
        assert_eq!(erc721.token.base_uri(), BASE_URI.to_string());
    }

    #[test]
    fn minting_by_not_an_owner() {
        // When deploy a contract with the initial supply.
        let erc721_env = setup();

        // Then minting a token by not an owner throws an error.
        assert_exception(NotAnOwner, || {
            test_env::set_caller(erc721_env.bob);
            let mut erc721 = Erc721TokenRef::at(erc721_env.token.address());
            erc721.mint(erc721_env.alice, U256::from(1));
        });
    }
}
