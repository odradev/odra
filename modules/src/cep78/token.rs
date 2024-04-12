use odra::{args::Maybe, casper_types::URef, prelude::*, Address};
use super::error::CEP78Error;

#[odra::module]
pub struct CEP78 {

}

// #[odra::module]
impl CEP78 {
    /// Initializes the module.
    pub fn init(&mut self) {
    }

    /// Exposes all variables that can be changed by managing account post
    /// installation. Meant to be called by the managing account (INSTALLER) post
    /// installation if a variable needs to be changed.
    /// By switching allow_minting to false we pause minting.
    pub fn set_variables(
        &mut self, 
        allow_minting: Maybe<bool>,
        contract_whitelist: Maybe<Vec<[u8; 32]>>,
        acl_whitelist: Maybe<Vec<Address>>,
        acl_package_mode: Maybe<bool>,
        package_operator_mode: Maybe<bool>,
        operator_burn_mode: Maybe<bool>,
    ) {
    
    }


    /// Mints a new token with provided metadata.
    /// Reverts with [CEP78Error::MintingIsPaused] error if `allow_minting` is false.
    /// When a token is minted the calling account is listed as its owner and the token is
    /// automatically assigned an `u64` ID equal to the current `number_of_minted_tokens`.
    /// Before minting the token, checks if `number_of_minted_tokens`
    /// exceeds the `total_token_supply`. If so, it reverts the minting with an error
    /// [CEP78Error::TokenSupplyDepleted]. The `mint` function also checks whether the calling account
    /// is the managing account (the installer) If not, and if `public_minting` is set to
    /// false, it reverts with the error [CEP78Error::InvalidAccount]. 
    /// After minting is successful the number_of_minted_tokens is incremented by one.
    pub fn mint(
        &mut self,
        token_owner: Address,
        token_metadata: String,
    ) -> (String, Address, String) {
        todo!()
    }


    /// Burns the token with provided `token_id` argument, after which it is no
    /// longer possible to transfer it.
    /// Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
    /// error [CEP78Error::InvalidTokenOwner]. If the token id is invalid (e.g. out of bounds) it reverts
    /// with error [CEP78Error::InvalidTokenIdentifier]. If the token is listed as already burnt we revert with
    /// error [CEP78Error::PreviouslyBurntToken]. If not the token is then registered as burnt.
    pub fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        todo!()
    }

    /// Transfers ownership of the token from one account to another.
    /// It looks up the owner of the supplied token_id arg. Reverts if the token is already burnt,
    /// `token_id` is invalid, or if caller is not owner nor an approved account nor operator.
    /// If token id is invalid it reverts with error [CEP78Error::InvalidTokenIdentifier].
    pub fn transfer(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>, source_key: Address, target_key: Address) -> (String, Address) {
        todo!()
    }

    /// Approves another token holder (an approved account) to transfer tokens. It
    /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    /// been burnt, or if caller tries to approve themselves as an approved account.
    pub fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        todo!()
    }

    /// Revokes an approved account to transfer tokens. It reverts
    /// if token_id is invalid, if caller is not the owner, if token has already
    /// been burnt, if caller tries to approve itself.
    pub fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        todo!()
    }

    /// Approves all tokens owned by the caller and future to another token holder
    /// (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
    /// owner, if caller tries to approve itself as an operator.
    pub fn set_approval_for_all(&mut self, approve_all: bool, operator: Address) {
        todo!()
    }

    /// Returns if an account is operator for a token owner
    pub fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool {
        todo!()
    }
    
    /// Returns the token owner given a token_id. It reverts if token_id
    /// is invalid. A burnt token still has an associated owner.
    pub fn owner_of(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        todo!()
    }
    
    /// Returns the approved account (if any) associated with the provided token_id
    /// Reverts if token has been burnt.
    pub fn get_approved(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Option<Address> {
        todo!()
    }

    /// Returns number of owned tokens associated with the provided token holder
    pub fn balance_of(&mut self, token_owner: Address) -> u64 {
        todo!()
    }

    /// Returns the metadata associated with the provided token_id
    pub fn get_token_metadata(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String {
        todo!()
    }

    /// Updates the metadata if valid.
    pub fn set_token_metadata(&mut self, token_metadata: String) {
        todo!()
    }
    
    /// This entrypoint will upgrade the contract from the 1_0 version to the
    /// 1_1 version. The contract will insert any addition dictionaries and
    /// sentinel values that were absent in the previous version of the contract.
    /// It will also perform the necessary data transformations of historical
    /// data if needed
    pub fn migrate(&mut self, nft_package_key: String) {
        todo!()
    }
    
    /// This entrypoint will allow NFT owners to update their receipts from
    /// the previous owned_tokens list model to the current pagination model
    /// scheme. Calling the entrypoint will return a list of receipt names
    /// alongside the dictionary addressed to the relevant pages.
    pub fn updated_receipts(&mut self) -> Vec<(String, Address)> {
        todo!()
    }

    /// This entrypoint allows users to register with a give CEP-78 instance,
    /// allocating the necessary page table to enable the reverse lookup
    /// functionality and allowing users to pay the upfront cost of allocation
    /// resulting in more stable gas costs when minting and transferring
    /// Note: This entrypoint MUST be invoked if the reverse lookup is enabled
    /// in order to own NFTs.
    pub fn register_owner(&mut self) -> (String, URef) {
        todo!()
    }
}
