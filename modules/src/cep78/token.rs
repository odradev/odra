#![allow(clippy::too_many_arguments)]
use super::{
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
    reverse_lookup::ReverseLookup,
    settings::Settings,
    whitelist::ACLWhitelist
};
use odra::{
    args::Maybe,
    casper_types::bytesrepr::ToBytes,
    prelude::*,
    Address, OdraError, SubModule, UnwrapOrRevert, Var
};

type MintReceipt = (String, Address, String);
type TransferReceipt = (String, Address);

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
#[odra::module]
pub struct Cep78 {
    data: SubModule<CollectionData>,
    metadata: SubModule<Metadata>,
    settings: SubModule<Settings>,
    whitelist: SubModule<ACLWhitelist>,
    reverse_lookup: SubModule<ReverseLookup>,
    transfer_filter_contract: Var<Address>
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
        nft_identifier_mode: NFTIdentifierMode,
        nft_metadata_kind: NFTMetadataKind,
        metadata_mutability: MetadataMutability,
        allow_minting: Maybe<bool>,
        minting_mode: Maybe<MintingMode>,
        holder_mode: Maybe<NFTHolderMode>,
        whitelist_mode: Maybe<WhitelistMode>,
        acl_white_list: Maybe<Vec<Address>>,
        json_schema: Maybe<String>,
        receipt_name: Maybe<String>,
        burn_mode: Maybe<BurnMode>,
        operator_burn_mode: Maybe<bool>,
        owner_reverse_lookup_mode: Maybe<OwnerReverseLookupMode>,
        events_mode: Maybe<EventsMode>,
        transfer_filter_contract_contract: Maybe<Address>,
        additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
        optional_metadata: Maybe<Vec<NFTMetadataKind>>
    ) {
        let installer = self.caller();
        self.data.init(
            collection_name,
            collection_symbol,
            total_token_supply,
            installer
        );
        self.settings.init(
            allow_minting.unwrap_or(true),
            minting_mode.clone().unwrap_or_default(),
            ownership_mode,
            nft_kind,
            holder_mode.unwrap_or_default(),
            burn_mode.unwrap_or_default(),
            events_mode.unwrap_or_default(),
            operator_burn_mode.unwrap_or_default()
        );

        self.reverse_lookup.init(
            owner_reverse_lookup_mode.clone().unwrap_or_default(),
            receipt_name.unwrap_or_default()
        );

        self.whitelist.init(
            acl_white_list.unwrap_or_default(),
            whitelist_mode.unwrap_or_default()
        );

        self.metadata.init(
            nft_metadata_kind,
            additional_required_metadata,
            optional_metadata,
            metadata_mutability,
            nft_identifier_mode,
            json_schema
        );

        if nft_identifier_mode == NFTIdentifierMode::Hash
            && metadata_mutability == MetadataMutability::Mutable
        {
            self.revert(CEP78Error::InvalidMetadataMutability)
        }

        if ownership_mode == OwnershipMode::Minter
            && minting_mode.unwrap_or_default() == MintingMode::Installer
            && owner_reverse_lookup_mode.unwrap_or_default() == OwnerReverseLookupMode::Complete
        {
            self.revert(CEP78Error::InvalidReportingMode)
        }

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
        token_metadata: String,
        token_hash: Maybe<String>
    ) -> MintReceipt {
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
                let hash = self.__env.hash(token_metadata.clone());
                base16::encode_lower(&hash)
            } else {
                optional_token_hash
            })
        };
        let token_id = token_identifier.to_string();

        self.metadata.update_or_revert(&token_metadata, &token_id);

        let token_owner = if self.is_transferable_or_assigned() {
            token_owner
        } else {
            caller
        };

        self.data.set_owner(&token_id, token_owner);
        self.data.set_issuer(&token_id, caller);

        if let NFTIdentifierMode::Hash = identifier_mode {
            self.reverse_lookup
                .insert_hash(minted_tokens_count, &token_identifier);
        }

        self.data.increment_counter(&token_owner);
        self.data.increment_number_of_minted_tokens();

        self.emit_ces_event(Mint::new(token_owner, token_id.clone(), token_metadata));

        self.reverse_lookup
            .on_mint(minted_tokens_count, token_owner, token_id)
    }

    /// Burns the token with provided `token_id` argument, after which it is no
    /// longer possible to transfer it.
    /// Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
    /// error [CEP78Error::InvalidTokenOwner]. If the token id is invalid (e.g. out of bounds) it reverts
    /// with error [CEP78Error::InvalidTokenIdentifier]. If the token is listed as already burnt we revert with
    /// error [CEP78Error::PreviouslyBurntToken]. If not the token is then registered as burnt.
    pub fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        self.ensure_burnable();

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

        self.ensure_not_burned(&token_id);
        self.data.mark_burnt(&token_id);
        self.data.decrement_counter(&token_owner);

        self.emit_ces_event(Burn::new(token_owner, token_id, caller));
    }

    /// Transfers ownership of the token from one account to another.
    /// It looks up the owner of the supplied token_id arg. Reverts if the token is already burnt,
    /// `token_id` is invalid, or if caller is not owner nor an approved account nor operator.
    /// If token id is invalid it reverts with error [CEP78Error::InvalidTokenIdentifier].
    pub fn transfer(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        source: Address,
        target: Address
    ) -> TransferReceipt {
        self.ensure_minter_or_assigned();

        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.ensure_not_burned(&token_id);
        self.ensure_owner(&token_id, &source);

        let caller = self.caller();
        let owner = self.owner_of_by_id(&token_id);
        let is_owner = owner == caller;

        let is_approved = !is_owner
            && match self.data.approved(&token_id) {
                Some(maybe_approved) => caller == maybe_approved,
                _ => false
            };

        let is_operator = if !is_owner && !is_approved {
            self.data.operator(source, caller)
        } else {
            false
        };

        if let Some(filter_contract) = self.transfer_filter_contract.get() {
            let result = TransferFilterContractContractRef::new(self.env(), filter_contract)
                .can_transfer(source, target, token_identifier.clone());

            if TransferFilterContractResult::DenyTransfer == result {
                self.revert(CEP78Error::TransferFilterContractDenied);
            }
        }

        if !is_owner && !is_approved && !is_operator {
            self.revert(CEP78Error::InvalidTokenOwner);
        }

        match self.data.owner_of(&token_id) {
            Some(token_actual_owner) => {
                if token_actual_owner != source {
                    self.revert(CEP78Error::InvalidTokenOwner)
                }
                self.data.set_owner(&token_id, target);
            }
            None => self.revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }

        self.data.decrement_counter(&source);
        self.data.increment_counter(&target);
        self.data.revoke(&token_id);

        let spender = if caller == owner { None } else { Some(caller) };
        self.emit_ces_event(Transfer::new(owner, spender, target, token_id));

        self.reverse_lookup
            .on_transfer(token_identifier, source, target)
    }

    /// Approves another token holder (an approved account) to transfer tokens. It
    /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    /// been burnt, or if caller tries to approve themselves as an approved account.
    pub fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        self.ensure_minter_or_assigned();

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
        self.ensure_minter_or_assigned();

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
        self.ensure_minter_or_assigned();
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
    pub fn metadata(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.metadata.get_or_revert(&token_identifier)
    }

    /// Updates the metadata if valid.
    pub fn set_token_metadata(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        updated_token_metadata: String
    ) {
        self.metadata
            .ensure_mutability(CEP78Error::ForbiddenMetadataUpdate);

        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.ensure_caller_is_owner(&token_id);
        self.metadata
            .update_or_revert(&updated_token_metadata, &token_id);

        self.emit_ces_event(MetadataUpdated::new(token_id, updated_token_metadata));
    }

    /// Returns number of owned tokens associated with the provided token holder
    pub fn balance_of(&mut self, token_owner: Address) -> u64 {
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
            .register_owner(token_owner, ownership_mode)
    }

    /*
    Test only getters
    */

    pub fn is_whitelisted(&self, address: &Address) -> bool {
        self.whitelist.is_whitelisted(address)
    }

    pub fn get_whitelist_mode(&self) -> WhitelistMode {
        self.whitelist.get_mode()
    }

    pub fn get_collection_name(&self) -> String {
        self.data.collection_name()
    }

    pub fn get_collection_symbol(&self) -> String {
        self.data.collection_symbol()
    }

    pub fn is_minting_allowed(&self) -> bool {
        self.settings.allow_minting()
    }

    pub fn is_operator_burn_mode(&self) -> bool {
        self.settings.operator_burn_mode()
    }

    pub fn get_total_supply(&self) -> u64 {
        self.data.total_token_supply()
    }

    pub fn get_minting_mode(&self) -> MintingMode {
        self.settings.minting_mode()
    }

    pub fn get_number_of_minted_tokens(&self) -> u64 {
        self.data.number_of_minted_tokens()
    }

    pub fn get_metadata_by_kind(
        &self,
        kind: NFTMetadataKind,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> String {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.metadata
            .get_metadata_by_kind(token_identifier.to_string(), &kind)
    }

    pub fn get_token_issuer(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.data.issuer(&token_identifier.to_string())
    }

    pub fn token_burned(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> bool {
        let token_identifier = self.token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        self.is_token_burned(&token_id)
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
    fn is_minter_or_assigned(&self) -> bool {
        matches!(
            self.ownership_mode(),
            OwnershipMode::Minter | OwnershipMode::Assigned
        )
    }

    #[inline]
    fn is_transferable_or_assigned(&self) -> bool {
        matches!(
            self.ownership_mode(),
            OwnershipMode::Transferable | OwnershipMode::Assigned
        )
    }

    #[inline]
    fn ensure_minter_or_assigned(&self) {
        if self.is_minter_or_assigned() {
            self.revert(CEP78Error::InvalidOwnershipMode)
        }
    }

    #[inline]
    fn token_identifier(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> TokenIdentifier {
        let env = self.env();
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&env))
        }
    }

    #[inline]
    fn checked_token_identifier(
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
            if token_identifier.get_index().unwrap_or_revert(&self.__env) >= number_of_minted_tokens
            {
                self.revert(CEP78Error::InvalidTokenIdentifier);
            }
        }
        token_identifier
    }

    #[inline]
    fn owner_of_by_id(&self, id: &String) -> Address {
        match self.data.owner_of(id) {
            Some(token_owner) => token_owner,
            None => self
                .env()
                .revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }
    }

    #[inline]
    fn is_token_burned(&self, token_id: &String) -> bool {
        self.data.is_burnt(token_id)
    }

    #[inline]
    fn ensure_owner(&self, token_id: &String, address: &Address) {
        let owner = self.owner_of_by_id(token_id);
        if address != &owner {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn ensure_caller_is_owner(&self, token_id: &String) {
        let owner = self.owner_of_by_id(token_id);
        if self.caller() != owner {
            self.revert(CEP78Error::InvalidTokenOwner);
        }
    }

    #[inline]
    fn ensure_not_burned(&self, token_id: &String) {
        if self.is_token_burned(token_id) {
            self.revert(CEP78Error::PreviouslyBurntToken);
        }
    }

    #[inline]
    fn ensure_not_caller(&self, address: Address) {
        if self.caller() == address {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn ensure_caller(&self, address: Address) {
        if self.caller() != address {
            self.revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn emit_ces_event<T: ToBytes>(&self, event: T) {
        let events_mode = self.settings.events_mode();
        if let EventsMode::CES = events_mode {
            self.env().emit_event(event);
        }
    }

    #[inline]
    fn ensure_burnable(&self) {
        if let BurnMode::NonBurnable = self.settings.burn_mode() {
            self.revert(CEP78Error::InvalidBurnMode)
        }
    }

    #[inline]
    fn ownership_mode(&self) -> OwnershipMode {
        self.settings.ownership_mode()
    }

    #[inline]
    fn verified_caller(&self) -> Address {
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