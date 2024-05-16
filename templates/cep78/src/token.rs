use odra::{args::Maybe, prelude::*, Address, SubModule};
use odra_modules::cep78::{
    modalities::{MetadataMutability, NFTIdentifierMode, NFTKind, NFTMetadataKind, OwnershipMode},
    token::{Cep78, MintReceipt, TransferReceipt},
};

/// A module definition. Each module struct consists Vars and Mappings
/// or/and another modules.
#[odra::module]
pub struct MyToken {
    /// A sub-module that implements the CEP-78 token standard.
    token: SubModule<Cep78>,
}

/// Module implementation.
///
/// To generate entrypoints,
/// an implementation block must be marked as #[odra::module].
#[odra::module]
impl MyToken {
    /// Initializes the contract with the given metadata and the default modalities.
    pub fn init(&mut self, collection_name: String, collection_symbol: String, total_supply: u64) {
        let receipt_name = format!("cep78_{}", collection_name);
        self.token.init(
            collection_name,
            collection_symbol,
            total_supply,
            OwnershipMode::Transferable,
            NFTKind::Digital,
            NFTIdentifierMode::Ordinal,
            NFTMetadataKind::Raw,
            MetadataMutability::Immutable,
            receipt_name,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
            Maybe::None,
        );
    }

    // Delegate all Cep78 functions to the token sub-module.
    delegate! {
        to self.token {
            /// Exposes all variables that can be changed by managing account post
            /// installation. Meant to be called by the managing account (`Installer`)
            /// if a variable needs to be changed.
            /// By switching `allow_minting` to false minting is paused.
            fn set_variables(
                &mut self,
                allow_minting: Maybe<bool>,
                acl_whitelist: Maybe<Vec<Address>>,
                operator_burn_mode: Maybe<bool>
            );

            /// Mints a new token with provided metadata.
            /// Reverts with [CEP78Error::MintingIsPaused] error if `allow_minting` is false.
            /// When a token is minted, the calling account is listed as its owner and the token is
            /// automatically assigned an `u64` ID equal to the current `number_of_minted_tokens`.
            /// Before minting, the token checks if `number_of_minted_tokens`
            /// exceeds the `total_token_supply`. If so, it reverts the minting with an error
            /// [CEP78Error::TokenSupplyDepleted]. The `mint` function also checks whether the calling account
            /// is the managing account (the installer) If not, and if `public_minting` is set to
            /// false, it reverts with the error [CEP78Error::InvalidAccount].
            /// After minting is successful the number_of_minted_tokens is incremented by one.
            fn mint(
                &mut self,
                token_owner: Address,
                token_meta_data: String,
                token_hash: Maybe<String>
            ) -> MintReceipt;

            /// Burns the token with provided `token_id` argument, after which it is no
            /// longer possible to transfer it.
            /// Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
            /// error [CEP78Error::InvalidTokenOwner]. If the token id is invalid (e.g. out of bounds) it reverts
            /// with error [CEP78Error::InvalidTokenIdentifier]. If the token is listed as already burnt we revert with
            /// error [CEP78Error::PreviouslyBurntToken]. If not the token is then registered as burnt.
            fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);

            /// Transfers ownership of the token from one account to another.
            /// It looks up the owner of the supplied token_id arg. Reverts if the token is already burnt,
            /// `token_id` is invalid, or if caller is not owner nor an approved account nor operator.
            /// If token id is invalid it reverts with error [CEP78Error::InvalidTokenIdentifier].
            fn transfer(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                source_key: Address,
                target_key: Address
            ) -> TransferReceipt;

            /// Approves another token holder (an approved account) to transfer tokens. It
            /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
            /// been burnt, or if caller tries to approve themselves as an approved account.
            fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>);

            /// Revokes an approved account to transfer tokens. It reverts
            /// if token_id is invalid, if caller is not the owner, if token has already
            /// been burnt, if caller tries to approve itself.
            fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);

            /// Approves all tokens owned by the caller and future to another token holder
            /// (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
            /// owner, if caller tries to approve itself as an operator.
            fn set_approval_for_all(&mut self, approve_all: bool, operator: Address);

            /// Returns if an account is operator for a token owner
            fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool;

            /// Returns the token owner given a token_id. It reverts if token_id
            /// is invalid. A burnt token still has an associated owner.
            fn owner_of(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address;

            /// Returns the approved account (if any) associated with the provided token_id
            /// Reverts if token has been burnt.
            fn get_approved(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>
            ) -> Option<Address>;

            /// Returns the metadata associated with the provided token_id
            fn metadata(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String;

            /// Updates the metadata if valid.
            fn set_token_metadata(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                token_meta_data: String
            );

            /// Returns number of owned tokens associated with the provided token holder
            fn balance_of(&mut self, token_owner: Address) -> u64;

            /// This entrypoint allows users to register with a give CEP-78 instance,
            /// allocating the necessary page table to enable the reverse lookup
            /// functionality and allowing users to pay the upfront cost of allocation
            /// resulting in more stable gas costs when minting and transferring
            /// Note: This entrypoint MUST be invoked if the reverse lookup is enabled
            /// in order to own NFTs.
            fn register_owner(&mut self, token_owner: Maybe<Address>) -> String;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn it_works() {
        let env = odra_test::env();
        // To test a module we need to deploy it. Autogenerated Autogenerated `MyTokenInitArgs`
        // implements `InitArgs` trait and `MyTokenHostRef` implements `Deployer` trait,
        // so we can use it to deploy the module.
        let init_args = MyTokenInitArgs {
            collection_name: "MyToken".to_string(),
            collection_symbol: "MT".to_string(),
            total_supply: 100,
        };
        assert!(MyTokenHostRef::try_deploy(&env, init_args).is_ok());
    }
}
