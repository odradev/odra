#![allow(unused_variables, missing_docs)]

use alloc::collections::BTreeMap;
use odra::{
    casper_types::{
        bytesrepr::{Bytes, ToBytes},
        U256
    },
    named_keys::{
        base64_encoded_key_value_storage, compound_key_value_storage, single_value_storage
    },
    prelude::*,
    ContractRef
};

/// Casper-compatible NFT interface
pub trait CEP95Interface {
    /// Returns a name of the NFT token/collection.
    fn name(&self) -> String;

    /// Returns a short symbol or abbreviation for the NFT token/collection.
    fn symbol(&self) -> String;

    /// Returns the number of NFTs owned by a given account or contract
    ///
    /// # Arguments
    /// owner - The account to query.
    ///
    /// # Returns
    /// The number of NFTs owned by the account.
    fn balance_of(&self, owner: Address) -> U256;

    /// Returns the owner of a specific NFT.
    ///
    /// # Arguments
    /// token_id - The ID of the NFT.
    ///
    /// # Returns
    /// The owner if it exists, else None.
    fn owner_of(&self, token_id: U256) -> Option<Address>;

    /// Performs a recipient check and transfers the ownership of an NFT.
    ///
    /// Reverts unless the contract caller is the current owner, an authorized
    /// operator, or the approved spender for this NFT. Reverts if `from` is not
    /// the current owner, or if `token_id` does not reference a valid NFT.
    /// Once ownership is updated and a `Transfer` event is emitted, the function
    /// checks whether `to` is a contract hash. If it is, the contract MUST call
    /// `on_cep95_received` on `to` and revert the entire transfer if that call
    /// is absent or returns any value other than `true`.
    ///
    /// # Arguments
    /// from - The current owner of the NFT.
    /// to - The new owner.
    /// token_id - The NFT ID.
    /// data - Optional payload to pass to a receiving contract.
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Option<Bytes>
    );

    /// Transfers the ownership of an NFT without checking the recipient contract.
    ///
    /// # Arguments
    /// from - The current owner of the NFT.
    /// to - The new owner.
    /// token_id - The NFT ID.
    fn transfer_from(&mut self, from: Address, to: Address, token_id: U256);

    /// Approves another account or contract to transfer a specific NFT.
    ///
    /// # Arguments
    /// spender - The account or contract that will be granted approval.
    /// token_id - The NFT ID.
    fn approve(&mut self, spender: Address, token_id: U256);

    /// Revokes approval for a specific NFT.
    ///
    /// # Arguments
    /// token_id - The NFT ID to revoke approval for.
    fn revoke_approval(&mut self, token_id: U256);

    /// Gets the approved account or contract for a specific NFT.
    ///
    /// # Arguments
    /// token_id - The ID of the NFT to check.
    ///
    /// # Returns
    /// Option<Address> - The approved spender account if one exists, else None.
    fn approved_for(&self, token_id: U256) -> Option<Address>;

    /// Enables operator approval for all of the caller's NFTs.
    ///
    /// # Arguments
    /// operator - The operator address to be approved.
    fn approve_for_all(&mut self, operator: Address);

    /// Revokes operator approval for all of the caller's NFTs.
    ///
    /// # Arguments
    /// operator - The operator address to be revoked.
    fn revoke_approval_for_all(&mut self, operator: Address);

    /// Checks if an operator is approved to manage all NFTs of the owner.
    ///
    /// # Arguments
    /// owner - The NFT owner's address.
    /// operator - The operator to check.
    ///
    /// # Returns
    /// True if the operator is approved for all NFTs, false otherwise.
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;

    /// Returns metadata for a given token ID.
    ///
    /// # Arguments
    /// token_id - The ID of the NFT.
    ///
    /// # Returns
    /// A vector of key-value pairs representing the metadata.
    fn token_metadata(&self, token_id: U256) -> Vec<(String, String)>;
}

#[odra::module]
/// Receiver interface
struct CEP95Receiver;

#[odra::module]
impl CEP95Receiver {
    /// Called after a `safe_transfer_from` completes its internal state update.
    /// MUST return `true` to signal acceptance; returning `false` or reverting
    /// causes the entire transfer to roll back.
    ///
    /// # Arguments
    /// operator - The account (EOA or contract) that invoked `safe_transfer_from`.
    /// from - The previous owner of `token_id`.
    /// token_id - The NFT being transferred.
    /// data - Opaque auxiliary data forwarded from the original call; may be `None` if no extra data was
    /// supplied.
    ///
    /// # Returns
    /// `true` to accept the NFT, anything else to reject.
    #[allow(dead_code)]
    pub fn on_cep95_received(
        &mut self,
        operator: &Address,
        from: &Address,
        token_id: &U256,
        data: &Option<Bytes>
    ) -> bool {
        // This is a placeholder implementation. In a real contract, you would
        // implement the logic to handle the received NFT here.
        // For example, you might want to store the token ID and data in your contract's state.
        // For now, we just return true to indicate acceptance.
        true
    }
}

const KEY_BALANCES: &str = "balances";
const KEY_NAME: &str = "name";
const KEY_SYMBOL: &str = "symbol";
const KEY_APPROVED: &str = "approvals";
const KEY_OPERATORS: &str = "operators";
const KEY_METADATA: &str = "metadata";
const KEY_OWNERS: &str = "owners";

single_value_storage!(Cep95Name, String, KEY_NAME, Error::ValueNotSet);
single_value_storage!(Cep95Symbol, String, KEY_SYMBOL, Error::ValueNotSet);
base64_encoded_key_value_storage!(Cep95Balances, KEY_BALANCES, Address, U256);
base64_encoded_key_value_storage!(Cep95Approvals, KEY_APPROVED, U256, Option<Address>);
compound_key_value_storage!(Cep95Operators, KEY_OPERATORS, Address, bool);
base64_encoded_key_value_storage!(Cep95Owners, KEY_OWNERS, U256, Option<Address>);
base64_encoded_key_value_storage!(Cep95Metadata, KEY_METADATA, U256, BTreeMap<String, String>);

/// Error enum for the CEP-95 contract.
#[odra::odra_error]
pub enum Error {
    /// The value is not set.
    ValueNotSet = 40_000,
    /// The transfer failed.
    TransferFailed = 40_001,
    /// The token ID is invalid.
    NotAnOwnerOrApproved = 40_002,
    /// The approval is set to the current owner.
    ApprovalToCurrentOwner = 40_003,
    /// The caller is the same as the operator.
    ApproveToCaller = 40_004,
    /// The token ID is invalid.
    InvalidTokenId = 40_005,
    /// The token with the given ID already exists.
    TokenAlreadyExists = 40_006
}

#[odra::event]
/// Emitted when an NFT is minted
pub struct Mint {
    /// The address of the recipient.
    pub to: Address,
    /// The ID of the minted token.
    pub token_id: U256
}

#[odra::event]
/// Emitted when an NFT is burned
pub struct Burn {
    /// The address of the owner.
    pub from: Address,
    /// The ID of the burned token.
    pub token_id: U256
}

#[odra::event]
/// Emitted when an NFT is transferred
pub struct Transfer {
    /// The address of the previous owner.
    pub from: Address,
    /// The address of the new owner.
    pub to: Address,
    /// The ID of the transferred token.
    pub token_id: U256
}

#[odra::event]
/// Emitted when a specific NFT is approved to an account/contract
pub struct Approval {
    /// The address of the owner.
    pub owner: Address,
    /// The address of the approved spender.
    pub spender: Address,
    /// The ID of the approved token.
    pub token_id: U256
}

#[odra::event]
/// Emitted when a specific NFT approval is revoked from an account/contract
pub struct RevokeApproval {
    /// The address of the owner.
    pub owner: Address,
    /// The address of the revoked spender.
    pub spender: Address,
    /// The ID of the revoked token.
    pub token_id: U256
}

#[odra::event]
/// Emitted when an operator is approved for all NFTs of an owner
pub struct ApprovalForAll {
    /// The address of the owner.
    pub owner: Address,
    /// The address of the operator.
    pub operator: Address
}

#[odra::event]
/// Emitted when an operator approval is revoked for all NFTs of an owner
pub struct RevokeApprovalForAll {
    /// The address of the owner.
    pub owner: Address,
    /// The address of the operator.
    pub operator: Address
}

#[odra::event]
/// Emitted whenever on-chain metadata for a token is created or updated
pub struct MetadataUpdate {
    /// The ID of the token.
    pub token_id: U256
}

#[odra::module(
    events = [
        Transfer,
        Approval,
        RevokeApproval,
        ApprovalForAll,
        RevokeApprovalForAll,
        Mint,
        Burn,
        MetadataUpdate
    ],
    errors = Error
)]
/// A module representing a CEP-95 standard.
pub struct Cep95 {
    /// A submodule for the token name.
    pub name: SubModule<Cep95Name>,
    /// A submodule for the token symbol.
    pub symbol: SubModule<Cep95Symbol>,
    /// A submodule for the token balances mapping.
    pub balances: SubModule<Cep95Balances>,
    /// A submodule for the token owners mapping.
    pub owners: SubModule<Cep95Owners>,
    /// A submodule for the token approvals mapping.
    pub approvals: SubModule<Cep95Approvals>,
    /// A submodule for the token operators mapping.
    pub operators: SubModule<Cep95Operators>,
    /// A submodule for the token metadata mapping.
    pub metadata: SubModule<Cep95Metadata>
}

#[odra::module]
impl CEP95Interface for Cep95 {
    fn name(&self) -> String {
        self.name.get()
    }

    fn symbol(&self) -> String {
        self.symbol.get()
    }

    fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(&owner).unwrap_or_default()
    }

    fn owner_of(&self, token_id: U256) -> Option<Address> {
        self.owners.get(&token_id).flatten()
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Option<Bytes>
    ) {
        self.transfer_from(from, to, token_id);
        if to.is_contract() {
            let mut receiver = CEP95ReceiverContractRef::new(self.env(), to);
            let caller = self.env().caller();
            let result = receiver.on_cep95_received(&caller, &from, &token_id, &data);
            if !result {
                self.env().revert(Error::TransferFailed);
            }
        }
    }

    fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        self.assert_exists(&token_id);

        let caller = self.env().caller();
        let owner = self
            .owner_of(token_id)
            .unwrap_or_revert_with(self, Error::ValueNotSet);
        // Only the owner or an approved spender can transfer the token.
        if (owner != from || owner != caller) && !self.is_approved_for_all(from, caller) {
            if let Some(approved) = self.approved_for(token_id) {
                if approved != caller {
                    self.env().revert(Error::NotAnOwnerOrApproved);
                }
            } else {
                self.env().revert(Error::NotAnOwnerOrApproved);
            }
        }

        self.raw_transfer_from(from, to, token_id);
    }

    fn approve(&mut self, spender: Address, token_id: U256) {
        let caller = self.env().caller();
        self.set_approve(token_id, Some(spender));
        self.env().emit_event(Approval {
            owner: caller,
            spender,
            token_id
        });
    }

    fn revoke_approval(&mut self, token_id: U256) {
        let spender = self
            .approved_for(token_id)
            .unwrap_or_revert_with(self, Error::ValueNotSet);
        self.set_approve(token_id, None);
        self.env().emit_event(RevokeApproval {
            owner: self.env().caller(),
            spender,
            token_id
        });
    }

    fn approved_for(&self, token_id: U256) -> Option<Address> {
        self.assert_exists(&token_id);
        self.approvals.get(&token_id).flatten()
    }

    fn approve_for_all(&mut self, operator: Address) {
        let caller = self.env().caller();
        self.set_approval_for_all(caller, operator, true);
        self.env().emit_event(ApprovalForAll {
            owner: caller,
            operator
        });
    }

    fn revoke_approval_for_all(&mut self, operator: Address) {
        let caller = self.env().caller();
        self.set_approval_for_all(caller, operator, false);
        self.env().emit_event(RevokeApprovalForAll {
            owner: caller,
            operator
        });
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operators.get_or_default(&owner, &operator)
    }

    fn token_metadata(&self, token_id: U256) -> Vec<(String, String)> {
        self.assert_exists(&token_id);
        self.metadata
            .get(&token_id)
            .unwrap_or_default()
            .into_iter()
            .collect()
    }
}

impl Cep95 {
    /// Initializes the module with a name and symbol.
    pub fn init(&mut self, name: String, symbol: String) {
        self.name.set(name);
        self.symbol.set(symbol);
    }

    #[inline]
    /// Asserts that the token ID exists.
    /// Reverts with `Error::InvalidTokenId` if it does not.
    pub fn assert_exists(&self, token_id: &U256) {
        if !self.exists(token_id) {
            self.env().revert(Error::InvalidTokenId);
        }
    }

    /// Checks if the token ID exists.
    #[inline]
    pub fn exists(&self, token_id: &U256) -> bool {
        self.owners.get(token_id).flatten().is_some()
    }

    /// Clears the approval for a specific token ID.
    #[inline]
    pub fn clear_approval(&mut self, token_id: &U256) {
        if self.approvals.get(token_id).is_some() {
            self.approvals.set(token_id, None);
        }
    }

    /// Mints a new NFT and assigns it to the specified address.
    pub fn mint(&mut self, to: Address, token_id: U256, metadata: Vec<(String, String)>) {
        if self.exists(&token_id) {
            self.env().revert(Error::TokenAlreadyExists);
        }

        self.balances.set(&to, self.balance_of(to) + 1);
        self.owners.set(&token_id, Some(to));
        self.set_metadata(token_id, metadata);

        self.env().emit_event(Mint { to, token_id });
    }

    /// Burns an NFT, removing it from the owner's balance and the contract.
    pub fn burn(&mut self, token_id: U256) {
        self.assert_exists(&token_id);
        let owner = self
            .owner_of(token_id)
            .unwrap_or_revert_with(self, Error::ValueNotSet);

        self.clear_approval(&token_id);
        self.balances.set(&owner, self.balance_of(owner) - 1);
        self.owners.set(&token_id, None);
        self.metadata.set(&token_id, Default::default());

        self.env().emit_event(Burn {
            from: owner,
            token_id
        });
    }

    /// Sets metadata for a specific token ID.
    /// Replaces any existing metadata.
    pub fn set_metadata(&mut self, token_id: U256, metadata: Vec<(String, String)>) {
        self.raw_update_metadata(token_id, metadata, BTreeMap::new());
    }

    /// Updates metadata for a specific token ID.
    /// If a key already exists, its value will be updated.
    /// If a key does not exist, it will be added.
    /// The remaining keys will be preserved.
    pub fn update_metadata(&mut self, token_id: U256, metadata: Vec<(String, String)>) {
        let current_metadata = self.metadata.get(&token_id).unwrap_or_default();
        self.raw_update_metadata(token_id, metadata, current_metadata);
    }

    /// Transfers an NFT from one address to another without checking the recipient contract.
    pub fn raw_transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        self.clear_approval(&token_id);
        self.balances.set(&from, self.balance_of(from) - 1);
        self.balances.set(&to, self.balance_of(to) + 1);
        self.owners.set(&token_id, Some(to));

        self.env().emit_event(Transfer { from, to, token_id });
    }

    fn raw_update_metadata(
        &mut self,
        token_id: U256,
        new_metadata: Vec<(String, String)>,
        mut current_metadata: BTreeMap<String, String>
    ) {
        self.assert_exists(&token_id);
        for (k, v) in new_metadata {
            current_metadata.insert(k, v);
        }
        self.metadata.set(&token_id, current_metadata);
        self.env().emit_event(MetadataUpdate { token_id });
    }

    #[inline]
    fn set_approve(&mut self, token_id: U256, spender: Option<Address>) {
        let owner = self
            .owner_of(token_id)
            .unwrap_or_revert_with(self, Error::ValueNotSet);
        let caller = self.env().caller();

        if Some(owner) == spender {
            self.env().revert(Error::ApprovalToCurrentOwner);
        }

        if caller != owner && !self.is_approved_for_all(owner, caller) {
            self.env().revert(Error::NotAnOwnerOrApproved);
        }

        self.approvals.set(&token_id, spender);
    }

    #[inline]
    fn set_approval_for_all(&mut self, caller: Address, operator: Address, approved: bool) {
        if caller == operator {
            self.env().revert(Error::ApproveToCaller)
        }

        self.operators.set(&caller, &operator, approved);
    }
}

mod utils {
    #![allow(dead_code)]
    
    use super::*;

    #[odra::module]
    pub(crate) struct BasicCep95 {
        token: SubModule<Cep95>
    }

    #[odra::module]
    impl BasicCep95 {
        /// Initializes the contract with the given parameters.
        pub fn init(&mut self, name: String, symbol: String) {
            self.token.init(name, symbol);
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

        pub fn mint(&mut self, to: Address, token_id: U256, metadata: Vec<(String, String)>) {
            self.token.mint(to, token_id, metadata);
        }

        pub fn burn(&mut self, token_id: U256) {
            self.token.burn(token_id);
        }

        pub fn set_metadata(&mut self, token_id: U256, metadata: Vec<(String, String)>) {
            self.token.set_metadata(token_id, metadata);
        }

        pub fn update_metadata(&mut self, token_id: U256, metadata: Vec<(String, String)>) {
            self.token.update_metadata(token_id, metadata);
        }
    }

    #[odra::module]
    pub(crate) struct NFTReceiver {
        #[allow(clippy::type_complexity)]
        last_call_data: Var<((Address, Address), (U256, Option<Bytes>))>
    }

    #[odra::module]
    impl NFTReceiver {
        #[allow(dead_code)]
        pub fn on_cep95_received(
            &mut self,
            operator: Address,
            from: Address,
            token_id: U256,
            data: Option<Bytes>
        ) -> bool {
            self.last_call_data
                .set(((operator, from), (token_id, data.clone())));
            true
        }

        #[allow(dead_code)]
        pub fn last_call_result(&self) -> ((Address, Address), (U256, Option<Bytes>)) {
            self.last_call_data.get().unwrap_or_revert(self)
        }
    }

    #[odra::module]
    pub(crate) struct RejectingNFTReceiver;

    #[odra::module]
    impl RejectingNFTReceiver {
        #[allow(dead_code)]
        pub fn on_cep95_received(
            &mut self,
            operator: Address,
            from: Address,
            token_id: U256,
            data: Option<Bytes>
        ) -> bool {
            false
        }
    }

    #[odra::module]
    pub(crate) struct BasicContract;

    #[odra::module]
    impl BasicContract {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cep95::utils::*;
    use odra::{
        host::{Deployer, HostEnv, NoArgs},
        Addressable, VmError
    };
    use odra_test;

    fn setup() -> (HostEnv, BasicCep95HostRef) {
        let env = odra_test::env();
        let cep95 = BasicCep95::try_deploy(
            &env,
            BasicCep95InitArgs {
                name: "TestToken".to_string(),
                symbol: "TT".to_string()
            }
        )
        .unwrap();
        (env, cep95)
    }

    #[test]
    fn test_deploy() {
        let env = odra_test::env();
        let cep95 = BasicCep95::try_deploy(
            &env,
            BasicCep95InitArgs {
                name: "TestToken".to_string(),
                symbol: "TT".to_string()
            }
        );
        assert!(cep95.is_ok());
    }

    #[test]
    fn test_name() {
        let (env, cep95) = setup();
        let name = cep95.name();
        assert_eq!(name, "TestToken");
    }

    #[test]
    fn test_symbol() {
        let (env, cep95) = setup();
        let symbol = cep95.symbol();
        assert_eq!(symbol, "TT");
    }

    #[test]
    fn test_mint() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        assert_eq!(cep95.balance_of(owner), U256::from(1));
        assert_eq!(cep95.owner_of(token_id), Some(owner));
        assert_eq!(
            cep95.token_metadata(token_id),
            vec![("key".to_string(), "value".to_string())]
        );
        assert!(env.emitted(cep95.address(), "Mint"));
    }

    #[test]
    fn test_minting_existing_token() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata.clone());

        let result = cep95.try_mint(owner, token_id, metadata);
        assert_eq!(result, Err(Error::TokenAlreadyExists.into()));
    }

    #[test]
    fn test_mint_many_tokens() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id1 = U256::from(1);
        let metadata1 = vec![("key1".to_string(), "value1".to_string())];
        cep95.mint(owner, token_id1, metadata1);

        let token_id2 = U256::from(2);
        let metadata2 = vec![("key2".to_string(), "value2".to_string())];
        cep95.mint(owner, token_id2, metadata2);

        assert_eq!(cep95.balance_of(owner), U256::from(2));
        assert_eq!(cep95.owner_of(token_id1), Some(owner));
        assert_eq!(cep95.owner_of(token_id2), Some(owner));
    }

    #[test]
    fn test_burn() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);
        cep95.burn(token_id);

        assert_eq!(cep95.balance_of(owner), U256::from(0));
        assert_eq!(cep95.owner_of(token_id), None);
        assert_eq!(
            cep95.try_token_metadata(token_id),
            Err(Error::InvalidTokenId.into())
        );
        assert!(env.emitted(cep95.address(), "Burn"));
    }

    #[test]
    fn test_burn_non_existing_token() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let result = cep95.try_burn(token_id);
        assert_eq!(result, Err(Error::InvalidTokenId.into()));
    }

    #[test]
    fn test_safe_transfer_to_receiver() {
        let (env, mut cep95) = setup();
        let nft_receiver = NFTReceiver::deploy(&env, NoArgs);
        let recipient = *nft_receiver.address();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        cep95.safe_transfer_from(owner, recipient, token_id, None);

        assert_eq!(cep95.balance_of(owner), U256::from(0));
        assert_eq!(cep95.balance_of(recipient), U256::from(1));
        assert_eq!(
            nft_receiver.last_call_result(),
            ((owner, owner), (token_id, None))
        );
    }

    #[test]
    fn test_safe_transfer_to_non_receiver() {
        let (env, mut cep95) = setup();
        let contract = BasicContract::deploy(&env, NoArgs);
        let recipient = *contract.address();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let result = cep95.try_safe_transfer_from(owner, recipient, token_id, None);
        assert_eq!(
            result,
            Err(OdraError::VmError(VmError::NoSuchMethod(
                "on_cep95_received".to_string()
            )))
        );

        assert_eq!(cep95.balance_of(owner), U256::from(1));
        assert_eq!(cep95.balance_of(recipient), U256::from(0));
    }

    #[test]
    fn test_safe_transfer_to_rejecting_receiver() {
        let (env, mut cep95) = setup();

        let owner = env.get_account(0);
        let contract = RejectingNFTReceiver::deploy(&env, NoArgs);
        let recipient = *contract.address();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let result = cep95.try_safe_transfer_from(owner, recipient, token_id, None);
        assert_eq!(result, Err(Error::TransferFailed.into()));

        assert_eq!(cep95.balance_of(owner), U256::from(1));
        assert_eq!(cep95.balance_of(recipient), U256::from(0));
    }

    #[test]
    fn test_transfer() {
        let (env, mut cep95) = setup();

        let owner = env.get_account(0);
        let recipient = env.get_account(10);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);
        cep95.transfer_from(owner, recipient, token_id);

        assert_eq!(cep95.balance_of(owner), U256::from(0));
        assert_eq!(cep95.balance_of(recipient), U256::from(1));
        assert!(env.emitted(cep95.address(), "Transfer"));
    }

    #[test]
    fn test_transfer_non_existing_token() {
        let (env, mut cep95) = setup();

        let owner = env.get_account(0);
        let recipient = env.get_account(10);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let result = cep95.try_transfer_from(owner, recipient, U256::from(2));
        assert_eq!(result, Err(Error::InvalidTokenId.into()));
    }

    #[test]
    fn test_transfer_from_non_owner() {
        let (env, mut cep95) = setup();

        let recipient = env.get_account(10);
        let non_owner = env.get_account(11);
        let owner = env.get_account(0);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let result = cep95.try_transfer_from(non_owner, recipient, token_id);
        assert_eq!(result, Err(Error::NotAnOwnerOrApproved.into()));
        assert_eq!(cep95.balance_of(owner), U256::from(1));
        assert_eq!(cep95.balance_of(recipient), U256::from(0));
    }

    #[test]
    fn test_approve() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let spender = env.get_account(10);
        cep95.approve(spender, token_id);

        assert_eq!(cep95.approved_for(token_id), Some(spender));
        assert!(env.emitted_event(
            &cep95,
            &Approval {
                owner,
                spender,
                token_id
            }
        ));
    }

    #[test]
    fn test_approve_by_non_owner() {
        let (env, mut cep95) = setup();
        let owner = env.get_account(0);
        let non_owner = env.get_account(11);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let spender = env.get_account(10);
        env.set_caller(non_owner);
        let result = cep95.try_approve(spender, token_id);
        assert_eq!(result, Err(Error::NotAnOwnerOrApproved.into()));
    }

    #[test]
    fn test_transfer_by_approved() {
        let (env, mut cep95) = setup();
        let owner = env.get_account(0);
        let recipient = env.get_account(10);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let spender = env.get_account(11);
        cep95.approve(spender, token_id);
        env.set_caller(spender);
        cep95.transfer_from(owner, recipient, token_id);

        assert_eq!(cep95.balance_of(owner), U256::from(0));
        assert_eq!(cep95.balance_of(recipient), U256::from(1));
        assert!(env.emitted_event(
            &cep95,
            &Transfer {
                from: owner,
                to: recipient,
                token_id
            }
        ));
    }

    #[test]
    fn test_approve_for_all() {
        let (env, mut cep95) = setup();
        let owner = env.caller();
        let operator = env.get_account(10);

        cep95.approve_for_all(operator);

        assert!(cep95.is_approved_for_all(owner, operator));
        assert!(env.emitted(cep95.address(), "ApprovalForAll"));
    }

    #[test]
    fn test_revoke_approval_for_all() {
        let (env, mut cep95) = setup();
        let owner = env.caller();
        let operator = env.get_account(10);

        cep95.approve_for_all(operator);
        cep95.revoke_approval_for_all(operator);

        assert!(!cep95.is_approved_for_all(owner, operator));
        assert!(env.emitted(cep95.address(), "RevokeApprovalForAll"));
    }

    #[test]
    fn test_approve_for_all_self() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let result = cep95.try_approve_for_all(owner);
        assert_eq!(result, Err(Error::ApproveToCaller.into()));
    }

    #[test]
    fn test_revoke_approval() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let spender = env.get_account(10);
        cep95.approve(spender, token_id);
        cep95.revoke_approval(token_id);

        assert_eq!(cep95.approved_for(token_id), None);
        assert!(env.emitted(cep95.address(), "RevokeApproval"));
    }

    #[test]
    fn revoke_non_existing_approval() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let result = cep95.try_revoke_approval(token_id);
        assert_eq!(result, Err(Error::ValueNotSet.into()));
    }

    #[test]
    fn test_revoke_approval_by_non_owner() {
        let (env, mut cep95) = setup();
        let owner = env.get_account(0);
        let non_owner = env.get_account(11);

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        let spender = env.get_account(10);
        cep95.approve(spender, token_id);
        env.set_caller(non_owner);
        let result = cep95.try_revoke_approval(token_id);
        assert_eq!(result, Err(Error::NotAnOwnerOrApproved.into()));
    }

    #[test]
    fn test_metadata() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id, metadata);

        assert_eq!(
            cep95.token_metadata(token_id),
            vec![("key".to_string(), "value".to_string())]
        );
    }

    #[test]
    fn test_transfer_by_operator() {
        let (env, mut cep95) = setup();
        let owner = env.caller();
        let recipient = env.get_account(10);
        let operator = env.get_account(11);

        let token_id1 = U256::from(1);
        let metadata = vec![("key".to_string(), "value".to_string())];
        cep95.mint(owner, token_id1, metadata.clone());

        cep95.approve_for_all(operator);
        let token_id2 = U256::from(2);
        cep95.mint(owner, token_id2, metadata);

        env.set_caller(operator);
        cep95.transfer_from(owner, recipient, token_id1);
        cep95.transfer_from(owner, recipient, token_id2);

        assert_eq!(cep95.balance_of(owner), U256::from(0));
        assert_eq!(cep95.balance_of(recipient), U256::from(2));
    }

    #[test]
    fn test_update_metadata() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![
            ("age".to_string(), "30".to_string()),
            ("name".to_string(), "Alice".to_string()),
        ];
        cep95.mint(owner, token_id, metadata);

        let new_metadata = vec![("name".to_string(), "Bob".to_string())];
        cep95.update_metadata(token_id, new_metadata);

        assert_eq!(
            cep95.token_metadata(token_id),
            vec![
                ("age".to_string(), "30".to_string()),
                ("name".to_string(), "Bob".to_string()),
            ]
        );
        assert!(env.emitted(cep95.address(), "MetadataUpdate"));
    }

    #[test]
    fn test_set_metadata() {
        let (env, mut cep95) = setup();
        let owner = env.caller();

        let token_id = U256::from(1);
        let metadata = vec![
            ("age".to_string(), "30".to_string()),
            ("name".to_string(), "Alice".to_string()),
        ];
        cep95.mint(owner, token_id, metadata);

        let new_metadata = vec![("name".to_string(), "Bob".to_string())];
        cep95.set_metadata(token_id, new_metadata);

        assert_eq!(
            cep95.token_metadata(token_id),
            vec![("name".to_string(), "Bob".to_string())]
        );
        assert!(env.emitted(cep95.address(), "MetadataUpdate"));
    }
}
