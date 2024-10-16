#![allow(clippy::too_many_arguments)]
use odra::named_keys::single_value_storage;

use super::{
    constants::{PREFIX_PAGE_DICTIONARY, TRANSFER_FILTER_CONTRACT},
    data::CollectionData,
    error::CEP78Error,
    events::{
        Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Mint, RevokedForAll,
        Transfer, VariablesSet
    },
    metadata::Metadata,
    modalities::{
        BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTKind, NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, TokenIdentifier,
        TransferFilterContractResult, WhitelistMode
    },
    reverse_lookup::{ReverseLookup, PAGE_SIZE},
    settings::Settings,
    utils,
    whitelist::ACLWhitelist
};
use odra::{
    args::Maybe, casper_event_standard::EventInstance, casper_types::bytesrepr::ToBytes,
    prelude::*, ContractRef
};

single_value_storage!(
    Cep78TransferFilterContract,
    Address,
    TRANSFER_FILTER_CONTRACT
);

/// CEP-78 is a standard for non-fungible tokens (NFTs) on the Casper network.
/// It defines a set of interfaces that allow for the creation, management, and
/// transfer of NFTs. The standard is designed to be flexible and modular, allowing
/// developers to customize the behavior of their NFTs to suit their specific needs.
/// The CEP-78 standard is inspired by the ERC-721 standard for NFTs on the Ethereum network.
/// The CEP-78 standard is designed to be simple and easy to use, while still providing
/// powerful features for developers to build on.
///
/// A list of mandatory init arguments:
/// - `collection_name`: The name of the NFT collection.
/// - `collection_symbol`: The symbol of the NFT collection.
/// - `total_token_supply`: The total number of tokens that can be minted in the collection.
/// - `ownership_mode`: The ownership mode of the collection. See [OwnershipMode] for more details.
/// - `nft_kind`: The kind of NFTs in the collection. See [NFTKind] for more details.
/// - `nft_identifier_mode`: The identifier mode of the NFTs in the collection. See [NFTIdentifierMode] for more details.
/// - `nft_metadata_kind`: The kind of metadata associated with the NFTs in the collection. See [NFTMetadataKind] for more details.
/// - `metadata_mutability`: The mutability of the metadata associated with the NFTs in the collection. See [MetadataMutability] for more details.
#[odra::module(
    version = "1.5.1",
    events = [Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Mint, RevokedForAll, Transfer, VariablesSet],
    errors = CEP78Error
)]
pub struct Cep78 {
    data: SubModule<CollectionData>,
    metadata: SubModule<Metadata>,
    settings: SubModule<Settings>,
    whitelist: SubModule<ACLWhitelist>,
    reverse_lookup: SubModule<ReverseLookup>,
    transfer_filter_contract: SubModule<Cep78TransferFilterContract>
}

#[odra::module]
impl Cep78 {
    /// Initializes the module.
    pub fn init(
        &mut self,
        collection_name: String,
        collection_symbol: String,
        total_token_supply: u64,
        ownership_mode: OwnershipMode,
        nft_kind: NFTKind,
        identifier_mode: NFTIdentifierMode,
        nft_metadata_kind: NFTMetadataKind,
        metadata_mutability: MetadataMutability,
        receipt_name: String,
        allow_minting: Maybe<bool>,
        minting_mode: Maybe<MintingMode>,
        holder_mode: Maybe<NFTHolderMode>,
        whitelist_mode: Maybe<WhitelistMode>,
        acl_whitelist: Maybe<Vec<Address>>,
        json_schema: Maybe<String>,
        burn_mode: Maybe<BurnMode>,
        operator_burn_mode: Maybe<bool>,
        owner_reverse_lookup_mode: Maybe<OwnerReverseLookupMode>,
        events_mode: Maybe<EventsMode>,
        transfer_filter_contract_contract: Maybe<Address>,
        additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
        optional_metadata: Maybe<Vec<NFTMetadataKind>>
    ) {
        let installer = self.caller();
        let minting_mode = minting_mode.unwrap_or_default();
        let owner_reverse_lookup_mode = owner_reverse_lookup_mode.unwrap_or_default();
        let acl_white_list = acl_whitelist.unwrap_or_default();
        let whitelist_mode = whitelist_mode.unwrap_or_default();
        let json_schema = json_schema.unwrap_or_default();
        let is_whitelist_empty = acl_white_list.is_empty();

        // Revert if minting mode is not ACL and acl list is not empty
        if MintingMode::Acl != minting_mode && !is_whitelist_empty {
            self.revert(CEP78Error::InvalidMintingMode)
        }

        // Revert if minting mode is ACL or holder_mode is contracts and acl list is locked and empty
        if MintingMode::Acl == minting_mode
            && is_whitelist_empty
            && WhitelistMode::Locked == whitelist_mode
        {
            self.revert(CEP78Error::EmptyACLWhitelist)
        }

        // NOTE: It is commented out to allow having mutable metadata with hash identifier.
        // NOTE: It's left for future reference.
        // if identifier_mode == NFTIdentifierMode::Hash
        //     && metadata_mutability == MetadataMutability::Mutable
        // {
        //     self.revert(CEP78Error::InvalidMetadataMutability)
        // }

        if ownership_mode == OwnershipMode::Minter
            && minting_mode == MintingMode::Installer
            && owner_reverse_lookup_mode == OwnerReverseLookupMode::Complete
        {
            self.revert(CEP78Error::InvalidReportingMode)
        }

        // Check if schema is missing before checking its validity
        if nft_metadata_kind == NFTMetadataKind::CustomValidated && json_schema.is_empty() {
            self.revert(CEP78Error::MissingJsonSchema)
        }

        // OwnerReverseLookup TransfersOnly mode should be Transferable
        if OwnerReverseLookupMode::TransfersOnly == owner_reverse_lookup_mode
            && OwnershipMode::Transferable != ownership_mode
        {
            self.revert(CEP78Error::OwnerReverseLookupModeNotTransferable)
        }

        if ownership_mode != OwnershipMode::Transferable
            && transfer_filter_contract_contract.is_some()
        {
            self.revert(CEP78Error::TransferFilterContractNeedsTransferableMode)
        }

        self.data.init(
            collection_name,
            collection_symbol,
            total_token_supply,
            installer
        );
        self.settings.init(
            allow_minting.unwrap_or(true),
            minting_mode,
            ownership_mode,
            nft_kind,
            holder_mode.unwrap_or_default(),
            burn_mode.unwrap_or_default(),
            events_mode.unwrap_or_default(),
            operator_burn_mode.unwrap_or_default()
        );

        self.reverse_lookup
            .init(owner_reverse_lookup_mode, receipt_name, total_token_supply);

        self.whitelist.init(acl_white_list.clone(), whitelist_mode);

        self.metadata.init(
            nft_metadata_kind,
            additional_required_metadata,
            optional_metadata,
            metadata_mutability,
            identifier_mode,
            json_schema
        );

        if let Maybe::Some(transfer_filter_contract_contract) = transfer_filter_contract_contract {
            self.transfer_filter_contract
                .set(transfer_filter_contract_contract);
        }
    }

    /// Exposes all variables that can be changed by managing account post
    /// installation. Meant to be called by the managing account (`Installer`)
    /// if a variable needs to be changed.
    /// By switching `allow_minting` to false minting is paused.
    pub fn set_variables(
        &mut self,
        allow_minting: Maybe<bool>,
        acl_whitelist: Maybe<Vec<Address>>,
        operator_burn_mode: Maybe<bool>
    ) {
        let installer = self.data.installer();
        self.ensure_caller(installer);

        if let Maybe::Some(allow_minting) = allow_minting {
            self.settings.set_allow_minting(allow_minting);
        }

        if let Maybe::Some(operator_burn_mode) = operator_burn_mode {
            self.settings.set_operator_burn_mode(operator_burn_mode);
        }

        self.whitelist.update(acl_whitelist);
        self.emit_ces_event(VariablesSet::new());
    }

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
    pub fn mint(
        &mut self,
        token_owner: Address,
        token_meta_data: String,
        token_hash: Maybe<String>
    ) {
        if !self.settings.allow_minting() {
            self.revert(CEP78Error::MintingIsPaused);
        }

        let total_token_supply = self.data.total_token_supply();
        let minted_tokens_count = self.data.number_of_minted_tokens();

        if minted_tokens_count >= total_token_supply {
            self.revert(CEP78Error::TokenSupplyDepleted);
        }

        let minting_mode = self.settings.minting_mode();
        let caller = self.verified_caller();

        if MintingMode::Installer == minting_mode {
            match caller {
                Address::Account(_) => {
                    let installer_account = self.data.installer();
                    if caller != installer_account {
                        self.revert(CEP78Error::InvalidMinter)
                    }
                }
                _ => self.revert(CEP78Error::InvalidKey)
            }
        }

        if MintingMode::Acl == minting_mode && !self.whitelist.is_whitelisted(&caller) {
            match caller {
                Address::Contract(_) => self.revert(CEP78Error::UnlistedContractHash),
                Address::Account(_) => self.revert(CEP78Error::InvalidMinter)
            }
        }

        let identifier_mode = self.metadata.get_identifier_mode();
        let optional_token_hash: String = token_hash.unwrap_or_default();
        let token_identifier: TokenIdentifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(minted_tokens_count),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(if optional_token_hash.is_empty() {
                let hash = self.__env.hash(token_meta_data.clone());
                base16::encode_lower(&hash)
            } else {
                optional_token_hash
            })
        };
        let token_id = token_identifier.to_string();

        // Check if token already exists.
        if self.token_exists_by_hash(&token_id) {
            self.revert(CEP78Error::DuplicateIdentifier)
        }

        self.metadata.update_or_revert(&token_meta_data, &token_id);

        let token_owner = if self.is_transferable_or_assigned() {
            token_owner
        } else {
            caller
        };

        self.data.set_owner(&token_id, token_owner);
        self.data.set_issuer(&token_id, caller);
        self.data.mark_not_burnt(&token_id);

        self.data.increment_counter(&token_owner);
        self.data.increment_number_of_minted_tokens();

        self.emit_ces_event(Mint::new(token_owner, token_id.clone(), token_meta_data));

        self.reverse_lookup.on_mint(&token_owner, &token_identifier)
    }

    /// Burns the token with provided `token_id` argument, after which it is no
    /// longer possible to transfer it.
    /// Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
    /// error [CEP78Error::InvalidTokenOwner]. If the token id is invalid (e.g. out of bounds) it reverts
    /// with error [CEP78Error::InvalidTokenIdentifier]. If the token is listed as already burnt we revert with
    /// error [CEP78Error::PreviouslyBurntToken]. If not the token is then registered as burnt.
    pub fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        let token_identifier = self.token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();

        let token_owner = self.owner_of_by_id(&token_id);
        let caller = self.__env.caller();

        let is_owner = token_owner == caller;
        let is_operator = if !is_owner {
            self.data.operator(token_owner, caller)
        } else {
            false
        };

        if !is_owner && !is_operator {
            self.revert(CEP78Error::InvalidTokenOwner)
        };

        // NOTE: Bellow code is almost the same as in `burn_token_unchecked`
        // function, but it is copied here to avoid checking owner twice.
        self.ensure_burnable();
        self.ensure_not_burned(&token_id);
        self.data.mark_burnt(&token_id);
        self.data.decrement_counter(&token_owner);
        self.data.decrement_number_of_minted_tokens();
        self.emit_ces_event(Burn::new(token_owner, token_id, caller));

        self.reverse_lookup.on_burn(&token_owner, &token_identifier);
    }

    /// Transfers ownership of the token from one account to another.
    /// It looks up the owner of the supplied token_id arg. Reverts if the token is already burnt,
    /// `token_id` is invalid, or if caller is not owner nor an approved account nor operator.
    /// If token id is invalid it reverts with error [CEP78Error::InvalidTokenIdentifier].
    pub fn transfer(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        source_key: Address,
        target_key: Address
    ) {
        self.ensure_not_minter_or_assigned();

        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.ensure_not_burned(&token_id);
        self.ensure_owner(&token_id, &source_key);

        let caller = self.caller();
        let owner = self.owner_of_by_id(&token_id);
        let is_owner = owner == caller;

        let is_approved = !is_owner
            && match self.data.approved(&token_id) {
                Some(maybe_approved) => caller == maybe_approved,
                _ => false
            };

        let is_operator = if !is_owner && !is_approved {
            self.data.operator(source_key, caller)
        } else {
            false
        };

        if let Some(filter_contract) = self.transfer_filter_contract.get() {
            let result = TransferFilterContractContractRef::new(self.env(), filter_contract)
                .can_transfer(source_key, target_key, token_identifier.clone());

            if TransferFilterContractResult::DenyTransfer == result {
                self.revert(CEP78Error::TransferFilterContractDenied);
            }
        }

        if !is_owner && !is_approved && !is_operator {
            self.revert(CEP78Error::InvalidTokenOwner);
        }

        match self.data.owner_of(&token_id) {
            Some(token_actual_owner) => {
                if token_actual_owner != source_key {
                    self.revert(CEP78Error::InvalidTokenOwner)
                }
            }
            None => self.revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }

        let spender = if caller == owner { None } else { Some(caller) };
        self.transfer_unchecked(token_id, source_key, spender, target_key);

        self.reverse_lookup
            .on_transfer(&token_identifier, &source_key, &target_key)
    }

    /// Approves another token holder (an approved account) to transfer tokens. It
    /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    /// been burnt, or if caller tries to approve themselves as an approved account.
    pub fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        self.ensure_not_minter_or_assigned();

        let caller = self.caller();
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();

        let owner = self.owner_of_by_id(&token_id);

        let is_owner = caller == owner;
        let is_operator = !is_owner && self.data.operator(owner, caller);

        if !is_owner && !is_operator {
            self.revert(CEP78Error::InvalidTokenOwner);
        }

        self.ensure_not_burned(&token_id);

        self.ensure_not_caller(spender);
        self.data.approve(&token_id, spender);
        self.emit_ces_event(Approval::new(owner, spender, token_id));
    }

    /// Revokes an approved account to transfer tokens. It reverts
    /// if token_id is invalid, if caller is not the owner, if token has already
    /// been burnt, if caller tries to approve itself.
    pub fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        self.ensure_not_minter_or_assigned();

        let caller = self.caller();
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();

        let owner = self.owner_of_by_id(&token_id);
        let is_owner = caller == owner;
        let is_operator = !is_owner && self.data.operator(owner, caller);

        if !is_owner && !is_operator {
            self.revert(CEP78Error::InvalidTokenOwner);
        }

        self.ensure_not_burned(&token_id);
        self.data.revoke(&token_id);

        self.emit_ces_event(ApprovalRevoked::new(owner, token_id));
    }

    /// Approves all tokens owned by the caller and future to another token holder
    /// (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
    /// owner, if caller tries to approve itself as an operator.
    pub fn set_approval_for_all(&mut self, approve_all: bool, operator: Address) {
        self.ensure_not_minter_or_assigned();
        self.ensure_not_caller(operator);

        let caller = self.caller();
        self.data.set_operator(caller, operator, approve_all);

        if let EventsMode::CES = self.settings.events_mode() {
            if approve_all {
                self.__env.emit_event(ApprovalForAll::new(caller, operator));
            } else {
                self.__env.emit_event(RevokedForAll::new(caller, operator));
            }
        }
    }

    /// Returns if an account is operator for a token owner
    pub fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool {
        self.data.operator(token_owner, operator)
    }

    /// Returns the token owner given a token_id. It reverts if token_id
    /// is invalid. A burnt token still has an associated owner.
    pub fn owner_of(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.owner_of_by_id(&token_identifier.to_string())
    }

    /// Returns the approved account (if any) associated with the provided token_id
    /// Reverts if token has been burnt.
    pub fn get_approved(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> Option<Address> {
        let token_identifier: TokenIdentifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();

        self.ensure_not_burned(&token_id);
        self.data.approved(&token_id)
    }

    /// Returns the metadata associated with the provided token_id
    pub fn metadata(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.metadata.get_or_revert(&token_identifier)
    }

    /// Updates the metadata if valid.
    pub fn set_token_metadata(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        token_meta_data: String
    ) {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.ensure_caller_is_owner(&token_id);
        self.set_token_metadata_unchecked(&token_id, token_meta_data);
    }

    /// Returns number of owned tokens associated with the provided token holder
    pub fn balance_of(&self, token_owner: Address) -> u64 {
        self.data.token_count(&token_owner)
    }

    /// This entrypoint allows users to register with a give CEP-78 instance,
    /// allocating the necessary page table to enable the reverse lookup
    /// functionality and allowing users to pay the upfront cost of allocation
    /// resulting in more stable gas costs when minting and transferring
    /// Note: This entrypoint MUST be invoked if the reverse lookup is enabled
    /// in order to own NFTs.
    pub fn register_owner(&mut self, token_owner: Maybe<Address>) -> String {
        let ownership_mode = self.ownership_mode();
        self.reverse_lookup
            .register_owner(token_owner, ownership_mode);
        // runtime::ret(CLValue::from_t((collection_name, package_uref)).unwrap_or_revert())
        "".to_string()
    }
}

impl Cep78 {
    #[inline]
    fn caller(&self) -> Address {
        self.__env.caller()
    }

    #[inline]
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.__env.revert(e)
    }

    #[inline]
    pub fn is_minter_or_assigned(&self) -> bool {
        matches!(
            self.ownership_mode(),
            OwnershipMode::Minter | OwnershipMode::Assigned
        )
    }

    #[inline]
    pub fn is_transferable_or_assigned(&self) -> bool {
        matches!(
            self.ownership_mode(),
            OwnershipMode::Transferable | OwnershipMode::Assigned
        )
    }

    #[inline]
    pub fn ensure_not_minter_or_assigned(&self) {
        if self.is_minter_or_assigned() {
            self.revert(CEP78Error::InvalidOwnershipMode)
        }
    }

    #[inline]
    pub fn token_identifier(
        &self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> TokenIdentifier {
        let env = self.env();
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&env))
        }
    }

    pub fn token_id(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String {
        let token_identifier = self.token_identifier(token_id, token_hash);
        token_identifier.to_string()
    }

    #[inline]
    pub fn checked_token_identifier(
        &self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> TokenIdentifier {
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        let token_identifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&self.__env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&self.__env))
        };

        let number_of_minted_tokens = self.data.number_of_minted_tokens();
        if let NFTIdentifierMode::Ordinal = identifier_mode {
            // Revert if token_id is out of bounds
            if token_identifier.get_index().unwrap_or_revert(self) >= number_of_minted_tokens {
                self.revert(CEP78Error::InvalidTokenIdentifier);
            }
        }
        token_identifier
    }

    #[inline]
    pub fn owner_of_by_id(&self, id: &str) -> Address {
        match self.data.owner_of(id) {
            Some(token_owner) => token_owner,
            None => self
                .env()
                .revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }
    }

    #[inline]
    pub fn is_token_burned(&self, token_id: &str) -> bool {
        self.data.is_burnt(token_id)
    }

    #[inline]
    pub fn ensure_owner(&self, token_id: &str, address: &Address) {
        let owner = self.owner_of_by_id(token_id);
        if address != &owner {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    pub fn ensure_caller_is_owner(&self, token_id: &str) {
        let owner = self.owner_of_by_id(token_id);
        if self.caller() != owner {
            self.revert(CEP78Error::InvalidTokenOwner);
        }
    }

    #[inline]
    pub fn ensure_not_burned(&self, token_id: &str) {
        if self.is_token_burned(token_id) {
            self.revert(CEP78Error::PreviouslyBurntToken);
        }
    }

    #[inline]
    pub fn ensure_not_caller(&self, address: Address) {
        if self.caller() == address {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    pub fn ensure_caller(&self, address: Address) {
        if self.caller() != address {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    pub fn emit_ces_event<T: ToBytes + EventInstance>(&self, event: T) {
        let events_mode = self.settings.events_mode();
        if let EventsMode::CES = events_mode {
            self.env().emit_event(event);
        }
    }

    #[inline]
    pub fn ensure_burnable(&self) {
        if let BurnMode::NonBurnable = self.settings.burn_mode() {
            self.revert(CEP78Error::InvalidBurnMode)
        }
    }

    #[inline]
    pub fn ownership_mode(&self) -> OwnershipMode {
        self.settings.ownership_mode()
    }

    #[inline]
    pub fn verified_caller(&self) -> Address {
        let holder_mode = self.settings.holder_mode();
        let caller = self.caller();

        match (caller, holder_mode) {
            (Address::Account(_), NFTHolderMode::Contracts)
            | (Address::Contract(_), NFTHolderMode::Accounts) => {
                self.revert(CEP78Error::InvalidHolderMode);
            }
            _ => caller
        }
    }

    // Check if token exists by hash.
    pub fn token_exists_by_hash(&self, token_id: &str) -> bool {
        self.data.owner_of(token_id).is_some() && !self.is_token_burned(token_id)
    }

    // Update metadata without ownership check.
    pub fn set_token_metadata_unchecked(&mut self, token_id: &String, token_meta_data: String) {
        self.metadata
            .ensure_mutability(CEP78Error::ForbiddenMetadataUpdate);
        self.metadata.update_or_revert(&token_meta_data, token_id);
        self.emit_ces_event(MetadataUpdated::new(
            String::from(token_id),
            token_meta_data
        ));
    }

    // Burn token without ownership check.
    pub fn burn_token_unchecked(&mut self, token_id: String, burner: Address) {
        self.ensure_burnable();
        let token_owner = self.owner_of_by_id(&token_id);
        self.ensure_not_burned(&token_id);
        self.data.mark_burnt(&token_id);
        self.data.decrement_counter(&token_owner);
        self.data.decrement_number_of_minted_tokens();
        self.emit_ces_event(Burn::new(token_owner, token_id, burner));
    }

    // Returns collection name.
    pub fn get_collection_name(&self) -> String {
        self.data.collection_name()
    }

    // Returns collection symbol.
    pub fn get_collection_symbol(&self) -> String {
        self.data.collection_symbol()
    }

    // Returns if address has admin rights.
    pub fn is_whitelisted(&self, address: &Address) -> bool {
        self.whitelist.is_whitelisted(address)
    }

    pub fn transfer_unchecked(
        &mut self,
        token_id: String,
        owner: Address,
        spender: Option<Address>,
        reciepient: Address
    ) {
        self.data.set_owner(&token_id, reciepient);
        self.data.decrement_counter(&owner);
        self.data.increment_counter(&reciepient);
        self.data.revoke(&token_id);

        self.emit_ces_event(Transfer::new(owner, spender, reciepient, token_id));
    }
}

#[odra::external_contract]
pub trait TransferFilterContract {
    fn can_transfer(
        &self,
        source_key: Address,
        target_key: Address,
        token_id: TokenIdentifier
    ) -> TransferFilterContractResult;
}

#[odra::module]
pub struct TestCep78 {
    token: SubModule<Cep78>
}

#[odra::module]
impl TestCep78 {
    delegate! {
        to self.token {
            fn init(
                &mut self,
                collection_name: String,
                collection_symbol: String,
                total_token_supply: u64,
                ownership_mode: OwnershipMode,
                nft_kind: NFTKind,
                identifier_mode: NFTIdentifierMode,
                nft_metadata_kind: NFTMetadataKind,
                metadata_mutability: MetadataMutability,
                receipt_name: String,
                allow_minting: Maybe<bool>,
                minting_mode: Maybe<MintingMode>,
                holder_mode: Maybe<NFTHolderMode>,
                whitelist_mode: Maybe<WhitelistMode>,
                acl_whitelist: Maybe<Vec<Address>>,
                json_schema: Maybe<String>,
                burn_mode: Maybe<BurnMode>,
                operator_burn_mode: Maybe<bool>,
                owner_reverse_lookup_mode: Maybe<OwnerReverseLookupMode>,
                events_mode: Maybe<EventsMode>,
                transfer_filter_contract_contract: Maybe<Address>,
                additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
                optional_metadata: Maybe<Vec<NFTMetadataKind>>
            );
            fn set_variables(
                &mut self,
                allow_minting: Maybe<bool>,
                acl_whitelist: Maybe<Vec<Address>>,
                operator_burn_mode: Maybe<bool>
            );
            fn mint(
                &mut self,
                token_owner: Address,
                token_meta_data: String,
                token_hash: Maybe<String>
            );
            fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn transfer(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                source_key: Address,
                target_key: Address
            );
            fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
            fn set_approval_for_all(&mut self, approve_all: bool, operator: Address);
            fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool;
            fn owner_of(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address;
            fn get_approved(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>
            ) -> Option<Address>;
            fn metadata(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String;
            fn set_token_metadata(
                &mut self,
                token_id: Maybe<u64>,
                token_hash: Maybe<String>,
                token_meta_data: String
            );
            fn balance_of(&self, token_owner: Address) -> u64;
            fn register_owner(&mut self, token_owner: Maybe<Address>) -> String;
            fn is_whitelisted(&self, address: &Address) -> bool;
        }
    }

    pub fn get_whitelist_mode(&self) -> WhitelistMode {
        self.token.whitelist.get_mode()
    }

    pub fn get_collection_name(&self) -> String {
        self.token.data.collection_name()
    }

    pub fn get_collection_symbol(&self) -> String {
        self.token.data.collection_symbol()
    }

    pub fn is_minting_allowed(&self) -> bool {
        self.token.settings.allow_minting()
    }

    pub fn is_operator_burn_mode(&self) -> bool {
        self.token.settings.operator_burn_mode()
    }

    pub fn get_total_supply(&self) -> u64 {
        self.token.data.total_token_supply()
    }

    pub fn get_minting_mode(&self) -> MintingMode {
        self.token.settings.minting_mode()
    }

    pub fn get_holder_mode(&self) -> NFTHolderMode {
        self.token.settings.holder_mode()
    }

    pub fn get_number_of_minted_tokens(&self) -> u64 {
        self.token.data.number_of_minted_tokens()
    }

    pub fn get_page(&self, page_number: u64) -> Vec<bool> {
        let env = self.env();
        let owner = env.caller();

        let owner_key = utils::address_to_key(&owner);
        let page_dict = format!("{PREFIX_PAGE_DICTIONARY}_{}", page_number);
        env.get_dictionary_value(page_dict, owner_key.as_bytes())
            .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidPageNumber)
    }

    pub fn get_page_by_token_id(&self, token_id: u64) -> Vec<bool> {
        let env = self.env();
        let owner = env.caller();
        let page_table_entry = token_id / PAGE_SIZE;

        let page_dict = format!("{PREFIX_PAGE_DICTIONARY}_{}", page_table_entry);
        let owner_key = utils::address_to_key(&owner);

        env.get_dictionary_value(page_dict, owner_key.as_bytes())
            .unwrap_or_revert_with(&env, CEP78Error::MissingPage)
    }

    pub fn get_page_by_token_hash(&self, token_hash: String) -> Vec<bool> {
        let identifier = TokenIdentifier::Hash(token_hash);
        let token_id = self
            .token
            .reverse_lookup
            .get_token_index_checked(&identifier);
        self.get_page_by_token_id(token_id)
    }

    pub fn get_page_table(&self) -> Vec<bool> {
        self.token
            .reverse_lookup
            .get_page_table(&self.__env.caller(), CEP78Error::MissingPage)
    }

    pub fn get_metadata_by_kind(
        &self,
        kind: NFTMetadataKind,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> String {
        let token_identifier = self.token.checked_token_identifier(token_id, token_hash);
        self.token
            .metadata
            .get_metadata_by_kind(token_identifier.to_string(), &kind)
    }

    pub fn get_token_issuer(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        let token_identifier = self.token.checked_token_identifier(token_id, token_hash);
        self.token.data.issuer(&token_identifier.to_string())
    }

    pub fn token_burned(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> bool {
        let token_identifier = self.token.token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.token.is_token_burned(&token_id)
    }
}
