//! A pluggable Odra module implementing Erc721 token with metadata and ownership.

use crate::access::Ownable;
use crate::erc721::erc721_base::Erc721Base;
use crate::erc721::events::Transfer;
use crate::erc721::extensions::erc721_metadata::{Erc721Metadata, Erc721MetadataExtension};
use crate::erc721::owned_erc721_with_metadata::OwnedErc721WithMetadata;
use crate::erc721::Erc721;
use crate::erc721_token::errors::Error;
use odra::prelude::*;
use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    Address, SubModule
};

/// The ERC721 token implementation.
///
/// It uses the [ERC721](Erc721Base) base implementation, the [ERC721 metadata](Erc721MetadataExtension) extension
/// and the [Ownable] module.
#[odra::module(events = [Transfer])]
pub struct Erc721Token {
    core: SubModule<Erc721Base>,
    metadata: SubModule<Erc721MetadataExtension>,
    ownable: SubModule<Ownable>
}

#[odra::module]
impl OwnedErc721WithMetadata for Erc721Token {
    fn init(&mut self, name: String, symbol: String, base_uri: String) {
        self.metadata.init(name, symbol, base_uri);
        self.ownable.init();
    }

    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn base_uri(&self) -> String {
        self.metadata.base_uri()
    }

    fn balance_of(&self, owner: &Address) -> U256 {
        self.core.balance_of(owner)
    }

    fn owner_of(&self, token_id: &U256) -> Address {
        self.core.owner_of(token_id)
    }

    fn safe_transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256) {
        self.core.safe_transfer_from(from, to, token_id);
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &U256,
        data: &Bytes
    ) {
        self.core
            .safe_transfer_from_with_data(from, to, token_id, data);
    }

    fn transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256) {
        self.core.transfer_from(from, to, token_id);
    }

    fn approve(&mut self, approved: &Option<Address>, token_id: &U256) {
        self.core.approve(approved, token_id);
    }

    fn set_approval_for_all(&mut self, operator: &Address, approved: bool) {
        self.core.set_approval_for_all(operator, approved);
    }

    fn get_approved(&self, token_id: &U256) -> Option<Address> {
        self.core.get_approved(token_id)
    }

    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool {
        self.core.is_approved_for_all(owner, operator)
    }

    fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership();
    }

    fn transfer_ownership(&mut self, new_owner: &Address) {
        self.ownable.transfer_ownership(new_owner);
    }

    fn owner(&self) -> Address {
        self.ownable.get_owner()
    }

    fn mint(&mut self, to: &Address, token_id: &U256) {
        self.ownable.assert_owner(&self.env().caller());

        if self.core.exists(token_id) {
            self.env().revert(Error::TokenAlreadyExists)
        }

        self.core.balances.add(to, U256::from(1));
        self.core.owners.set(token_id, Some(*to));
    }

    fn burn(&mut self, token_id: &U256) {
        self.core.assert_exists(token_id);
        self.ownable.assert_owner(&self.env().caller());

        let owner = self.core.owner_of(token_id);
        let balance = self.core.balance_of(&owner);
        self.core.balances.set(&owner, balance - U256::from(1));
        self.core.owners.set(token_id, None);
        self.core.clear_approval(token_id);

        self.env().emit_event(Transfer {
            from: Some(owner),
            to: None,
            token_id: *token_id
        });
    }
}

/// Erc721 errors.
pub mod errors {
    use odra::OdraError;

    /// Erc721 errors.
    #[derive(OdraError)]
    pub enum Error {
        /// Token with a given id already exists.
        TokenAlreadyExists = 35_000
    }
}

#[cfg(test)]
mod tests {
    use super::{Erc721TokenHostRef, Erc721TokenInitArgs};
    use crate::access::errors::Error as AccessError;
    use crate::erc20::{Erc20HostRef, Erc20InitArgs};
    use crate::erc721::errors::Error::{InvalidTokenId, NotAnOwnerOrApproved};
    use crate::erc721_receiver::events::Received;
    use crate::erc721_receiver::Erc721ReceiverHostRef;
    use crate::erc721_token::errors::Error::TokenAlreadyExists;
    use odra::host::{Deployer, HostEnv, HostRef, NoArgs};
    use odra::prelude::*;
    use odra::{casper_types::U256, Address, OdraError, VmError};

    const NAME: &str = "PlascoinNFT";
    const SYMBOL: &str = "PLSNFT";
    const BASE_URI: &str = "https://plascoin.org/";

    struct TokenEnv {
        env: HostEnv,
        token: Erc721TokenHostRef,
        alice: Address,
        bob: Address,
        carol: Address
    }

    fn setup() -> TokenEnv {
        let env = odra_test::env();
        TokenEnv {
            env: env.clone(),
            token: Erc721TokenHostRef::deploy(
                &env,
                Erc721TokenInitArgs {
                    name: NAME.to_string(),
                    symbol: SYMBOL.to_string(),
                    base_uri: BASE_URI.to_string()
                }
            ),
            alice: env.get_account(1),
            bob: env.get_account(2),
            carol: env.get_account(3)
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
    fn minting_same_id() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then minting the same token again throws an error.
        let err = erc721_env
            .token
            .try_mint(erc721_env.alice, U256::from(1))
            .unwrap_err();
        assert_eq!(err, TokenAlreadyExists.into());
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
        assert_eq!(
            erc721_env.token.try_owner_of(U256::from(1)).unwrap_err(),
            InvalidTokenId.into()
        );
    }

    #[test]
    fn approve() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And approve Bob to transfer the token.
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env
            .token
            .approve(Some(erc721_env.bob), U256::from(1));

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
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env
            .token
            .approve(Some(erc721_env.bob), U256::from(1));

        // And cancel the approval.
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env.token.approve(None, U256::from(1));

        // Then Bob is not approved to transfer the token.
        assert_eq!(erc721_env.token.get_approved(U256::from(1)), None);
    }

    #[test]
    fn approve_non_existing_token() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // Then approving a non existing token throws an error.
        assert_eq!(
            erc721_env
                .token
                .try_approve(Some(erc721_env.bob), U256::from(1))
                .unwrap_err(),
            InvalidTokenId.into()
        );
    }

    #[test]
    fn approve_not_owned_token() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then approving a token that is not owned by the caller throws an error.
        assert_eq!(
            erc721_env
                .token
                .try_approve(Some(erc721_env.bob), U256::from(1))
                .unwrap_err(),
            NotAnOwnerOrApproved.into()
        );
    }

    #[test]
    fn set_an_operator() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And set Bob as an operator.
        erc721_env.env.set_caller(erc721_env.alice);
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
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, true);

        // And cancel Bob as an operator.
        erc721_env.env.set_caller(erc721_env.alice);
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
        erc721_env.env.set_caller(erc721_env.alice);
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
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env
            .token
            .approve(Some(erc721_env.bob), U256::from(1));

        // And transfer the token to Carol.
        erc721_env.env.set_caller(erc721_env.bob);
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
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env.token.set_approval_for_all(erc721_env.bob, true);

        // And transfer the token to Carol.
        erc721_env.env.set_caller(erc721_env.bob);
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
        erc721_env.env.set_caller(erc721_env.bob);
        let err = erc721_env
            .token
            .try_transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1))
            .unwrap_err();
        assert_eq!(err, NotAnOwnerOrApproved.into());
    }

    #[test]
    fn transferring_invalid_nft() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // Then transferring a token that does not exist throws an error.
        let err = erc721_env
            .token
            .try_transfer_from(erc721_env.alice, erc721_env.carol, U256::from(1))
            .unwrap_err();
        assert_eq!(err, InvalidTokenId.into());
    }

    #[test]
    fn safe_transfer() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        assert!(erc721_env.token.address().is_contract());
        assert!(!erc721_env.alice.is_contract());

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And safe transfer the token to Bob.
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env
            .token
            .safe_transfer_from(erc721_env.alice, erc721_env.bob, U256::from(1));

        // Then the owner of the token is Bob.
        assert_eq!(erc721_env.token.owner_of(U256::from(1)), erc721_env.bob);
    }

    #[test]
    fn safe_transfer_to_contract_which_does_not_support_nft() {
        // When deploy a contract with the initial supply
        let mut erc721_env = setup();
        // And another contract which does not support nfts
        let erc20 = Erc20HostRef::deploy(
            &erc721_env.env,
            Erc20InitArgs {
                name: "PLASCOIN".to_string(),
                symbol: "PLS".to_string(),
                decimals: 10,
                initial_supply: None
            }
        );

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then safe transfer the token to the contract which does not support nfts throws an error.
        erc721_env.env.set_caller(erc721_env.alice);

        assert_eq!(
            Err(OdraError::VmError(VmError::NoSuchMethod(
                "on_erc721_received".to_string()
            ))),
            erc721_env.token.try_safe_transfer_from(
                erc721_env.alice,
                *erc20.address(),
                U256::from(1)
            )
        );
    }

    #[test]
    fn safe_transfer_to_contract_which_supports_nft() {
        // When deploy a contract with the initial supply
        let mut erc721_env = setup();
        // And another contract which does not support nfts
        let receiver = Erc721ReceiverHostRef::deploy(&erc721_env.env, NoArgs);

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And transfer the token to the contract which does support nfts
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env
            .token
            .safe_transfer_from(erc721_env.alice, *receiver.address(), U256::from(1));

        // Then the owner of the token is the contract
        assert_eq!(
            erc721_env.token.owner_of(U256::from(1)),
            *receiver.address()
        );
        // And the receiver contract is aware of the transfer
        erc721_env.env.emitted_event(
            receiver.address(),
            &Received {
                operator: Some(erc721_env.alice),
                from: Some(erc721_env.alice),
                token_id: U256::from(1),
                data: None
            }
        );
    }

    #[test]
    fn safe_transfer_to_contract_with_data() {
        // When deploy a contract with the initial supply
        let mut erc721_env = setup();
        // And another contract which does not support nfts
        let receiver = Erc721ReceiverHostRef::deploy(&erc721_env.env, NoArgs);

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And transfer the token to the contract which does support nfts
        erc721_env.env.set_caller(erc721_env.alice);
        erc721_env.token.safe_transfer_from_with_data(
            erc721_env.alice,
            *receiver.address(),
            U256::from(1),
            b"data".to_vec().into()
        );

        // Then the owner of the token is the contract
        assert_eq!(
            erc721_env.token.owner_of(U256::from(1)),
            receiver.address().clone()
        );
        // And the receiver contract is aware of the transfer
        erc721_env.env.emitted_event(
            receiver.address(),
            &Received {
                operator: Some(erc721_env.alice),
                from: Some(erc721_env.alice),
                token_id: U256::from(1),
                data: Some(b"data".to_vec().into())
            }
        );
    }

    #[test]
    fn burn() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // And burn the token.
        erc721_env.token.burn(U256::from(1));

        // Then the owner of throws an error.
        let err = erc721_env.token.try_owner_of(U256::from(1)).unwrap_err();
        assert_eq!(err, InvalidTokenId.into());
    }

    #[test]
    fn burn_error() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // And mint a token to Alice.
        erc721_env.token.mint(erc721_env.alice, U256::from(1));

        // Then burn the token as Alice errors out
        erc721_env.env.set_caller(erc721_env.alice);
        let err = erc721_env.token.try_burn(U256::from(1)).unwrap_err();
        assert_eq!(err, AccessError::CallerNotTheOwner.into());
    }

    #[test]
    fn burn_non_existing_nft() {
        // When deploy a contract with the initial supply.
        let mut erc721_env = setup();

        // Then burning a token that does not exist throws an error.
        let err = erc721_env.token.try_burn(U256::from(1)).unwrap_err();
        assert_eq!(err, InvalidTokenId.into());
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
        let mut erc721_env = setup();

        // Then minting a token by not an owner throws an error.
        erc721_env.env.set_caller(erc721_env.bob);
        let err = erc721_env
            .token
            .try_mint(erc721_env.alice, U256::from(1))
            .unwrap_err();
        assert_eq!(err, AccessError::CallerNotTheOwner.into());
    }
}
