use super::{
    collection_info::CollectionInfo,
    constants,
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
    pagination::{Pagination, PAGE_SIZE},
    reverse_lookup::ReverseLookup,
    settings::Settings,
    utils,
    whitelist::ACLWhitelist
};
use odra::{
    args::Maybe,
    casper_types::{bytesrepr::ToBytes, URef},
    prelude::*,
    Address, Mapping, Sequence, SubModule, UnwrapOrRevert, Var
};

#[odra::module]
pub struct CEP78 {
    whitelist: SubModule<ACLWhitelist>,
    metadata: SubModule<Metadata>,
    reverse_lookup: SubModule<ReverseLookup>,
    pagination: SubModule<Pagination>,
    info: SubModule<CollectionInfo>,
    settings: SubModule<Settings>,
    owners: Mapping<String, Address>,
    issuers: Mapping<String, Address>,
    approved: Mapping<String, Option<Address>>,
    token_count: Mapping<Address, u64>,
    burnt_tokens: Mapping<String, ()>,
    operators: Mapping<(Address, Address), bool>,
    receipt_name: Var<String>
}

#[odra::module]
impl CEP78 {
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
        transfer_filter_contract_contract_key: Maybe<Address>,
        additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
        optional_metadata: Maybe<Vec<NFTMetadataKind>>
    ) {
        let installer = self.env().caller();
        self.info.init(
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

        self.reverse_lookup
            .init(owner_reverse_lookup_mode.clone().unwrap_or_default());
        self.receipt_name.set(receipt_name.unwrap_or_default());

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
            self.env().revert(CEP78Error::InvalidMetadataMutability)
        }

        if ownership_mode == OwnershipMode::Minter
            && minting_mode.unwrap_or_default() == MintingMode::Installer
            && owner_reverse_lookup_mode.unwrap_or_default() == OwnerReverseLookupMode::Complete
        {
            self.env().revert(CEP78Error::InvalidReportingMode)
        }
    }

    /// Exposes all variables that can be changed by managing account post
    /// installation. Meant to be called by the managing account (INSTALLER) post
    /// installation if a variable needs to be changed.
    /// By switching allow_minting to false we pause minting.
    pub fn set_variables(
        &mut self,
        allow_minting: Maybe<bool>,
        acl_whitelist: Maybe<Vec<Address>>,
        operator_burn_mode: Maybe<bool>
    ) {
        let installer = self.info.installer();
        // Only the installing account can change the mutable variables.
        self.ensure_caller(installer);

        if let Maybe::Some(allow_minting) = allow_minting {
            self.settings.set_allow_minting(allow_minting);
        }

        if let Maybe::Some(operator_burn_mode) = operator_burn_mode {
            self.settings.set_operator_burn_mode(operator_burn_mode);
        }

        self.whitelist.update_addresses(acl_whitelist);
        self.emit_ces_event(VariablesSet::new());
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
        token_hash: Maybe<String>
    ) -> (String, Address, String) {
        // The contract owner can toggle the minting behavior on and off over time.
        // The contract is toggled on by default.
        let allow_minting = self.settings.allow_minting();

        // If contract minting behavior is currently toggled off we revert.
        if !allow_minting {
            self.env().revert(CEP78Error::MintingIsPaused);
        }

        let total_token_supply = self.info.total_token_supply();

        // The minted_tokens_count is the number of minted tokens so far.
        let minted_tokens_count = self.info.number_of_minted_tokens();

        // Revert if the token supply has been exhausted.
        if minted_tokens_count >= total_token_supply {
            self.env().revert(CEP78Error::TokenSupplyDepleted);
        }

        let minting_mode: MintingMode = self.settings.minting_mode();

        let caller = self.verified_caller();

        // Revert if minting is private and caller is not installer.
        if MintingMode::Installer == minting_mode {
            match caller {
                Address::Account(_) => {
                    let installer_account = self.info.installer();
                    // Revert if private minting is required and caller is not installer.
                    if caller != installer_account {
                        self.env().revert(CEP78Error::InvalidMinter)
                    }
                }
                _ => self.env().revert(CEP78Error::InvalidKey)
            }
        }

        // Revert if minting is acl and caller is not whitelisted.
        if MintingMode::Acl == minting_mode {
            let is_whitelisted = self.whitelist.is_whitelisted(&caller);
            if !is_whitelisted {
                match caller {
                    Address::Contract(_) => self.env().revert(CEP78Error::UnlistedContractHash),
                    Address::Account(_) => self.env().revert(CEP78Error::InvalidMinter)
                }
            }
        }

        let identifier_mode = self.metadata.get_identifier_mode();

        let optional_token_hash: String = token_hash.unwrap_or_default();
        let token_identifier: TokenIdentifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(minted_tokens_count),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(if optional_token_hash.is_empty() {
                base16::encode_lower(&self.env().hash(token_metadata.clone()))
            } else {
                optional_token_hash
            })
        };

        self.metadata
            .update_or_revert(&token_metadata, &token_identifier);

        // The contract's ownership behavior (determined at installation) determines,
        // who owns the NFT we are about to mint.()
        let token_owner_key =
            if let OwnershipMode::Assigned | OwnershipMode::Transferable = self.ownership_mode() {
                token_owner
            } else {
                caller
            };

        let id = token_identifier.to_string();
        self.owners.set(&id, token_owner_key);
        self.issuers.set(&id, caller);

        if let NFTIdentifierMode::Hash = identifier_mode {
            // Update the forward and reverse trackers
            self.reverse_lookup
                .insert_hash(minted_tokens_count, &token_identifier);
        }

        //Increment the count of owned tokens.
        self.token_count.add(&token_owner_key, 1);

        // Increment number_of_minted_tokens by one
        self.info.increment_number_of_minted_tokens();

        // Emit Mint event.
        self.emit_ces_event(Mint::new(
            token_owner_key,
            token_identifier.clone(),
            token_metadata.clone()
        ));

        if let OwnerReverseLookupMode::Complete = self.reverse_lookup.get_mode() {
            let (page_table_entry, page_uref) = self.pagination.add_page_entry_and_page_record(
                minted_tokens_count,
                &token_owner_key,
                true
            );
            let receipt_name = self.receipt_name.get_or_default();
            let receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // TODO: Implement the following
            // let receipt_address = Key::dictionary(page_uref, owned_tokens_item_key.as_bytes());
            let token_identifier_string = token_identifier.to_string();
            // should not return `token_owner`
            return (receipt_string, token_owner, token_identifier_string);
        }
        (id, token_owner_key, token_metadata)
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

        let token_owner = self.owner_of_by_id(&token_identifier);
        let caller = self.env().caller();

        // Check if caller is owner
        let is_owner = token_owner == caller;

        // Check if caller is operator to execute burn
        let is_operator = if !is_owner {
            self.operators.get_or_default(&(token_owner, caller))
        } else {
            false
        };

        // Revert if caller is not token_owner nor operator for the owner
        if !is_owner && !is_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner)
        };

        // It makes sense to keep this token as owned by the caller. It just happens that the caller
        // owns a burnt token. That's all. Similarly, we should probably also not change the
        // owned_tokens dictionary.
        self.ensure_not_burned(&token_identifier);

        // Mark the token as burnt by adding the token_id to the burnt tokens dictionary.
        self.burnt_tokens.set(&token_identifier.to_string(), ());
        self.token_count.subtract(&token_owner, 1);

        // Emit Burn event.
        self.emit_ces_event(Burn::new(token_owner, token_identifier, caller));
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
    ) -> (String, Address) {
        // If we are in minter or assigned mode we are not allowed to transfer ownership of token, hence
        // we revert.
        self.ensure_minter_or_assigned();

        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        let token_id = token_identifier.to_string();
        // We assume we cannot transfer burnt tokens
        self.ensure_not_burned(&token_identifier);
        self.ensure_not_owner(&token_identifier, &source_key);

        let caller = self.env().caller();

        let owner = self.owner_of_by_id(&token_identifier);
        // Check if caller is owner
        let is_owner = owner == caller;

        // Check if caller is approved to execute transfer
        let is_approved = !is_owner
            && match self.approved.get(&token_id) {
                Some(Some(maybe_approved)) => caller == maybe_approved,
                Some(None) | None => false
            };

        // Check if caller is operator to execute transfer
        let is_operator = if !is_owner && !is_approved {
            self.operators.get_or_default(&(source_key, caller))
        } else {
            false
        };

        if let Some(filter_contract) = utils::get_transfer_filter_contract() {
            let result = TransferFilterContractContractRef::new(self.env(), filter_contract)
                .can_transfer(source_key, target_key, token_identifier.clone());

            if TransferFilterContractResult::DenyTransfer == result {
                self.env().revert(CEP78Error::TransferFilterContractDenied);
            }
        }

        // Revert if caller is not owner nor approved nor an operator.
        if !is_owner && !is_approved && !is_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }

        // Updated token_owners dictionary. Revert if token_owner not found.
        match self.owners.get(&token_id) {
            Some(token_actual_owner) => {
                if token_actual_owner != source_key {
                    self.env().revert(CEP78Error::InvalidTokenOwner)
                }
                self.owners.set(&token_identifier.to_string(), target_key);
            }
            None => self
                .env()
                .revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }

        self.token_count.subtract(&source_key, 1);
        self.token_count.add(&target_key, 1);

        self.approved.set(&token_id, Option::<Address>::None);

        let spender = if caller == owner { None } else { Some(caller) };
        self.emit_ces_event(Transfer::new(
            owner,
            spender,
            target_key,
            token_identifier.clone()
        ));

        let reporting_mode = self.reverse_lookup.get_mode();

        if let OwnerReverseLookupMode::Complete | OwnerReverseLookupMode::TransfersOnly =
            reporting_mode
        {
            // Update to_account owned_tokens. Revert if owned_tokens list is not found
            let tokens_count = self.reverse_lookup.get_token_index(&token_identifier);
            if OwnerReverseLookupMode::TransfersOnly == reporting_mode {
                self.pagination
                    .add_page_entry_and_page_record(tokens_count, &source_key, false);
            }

            let (page_table_entry, page_uref) = self.pagination.update_page_entry_and_page_record(
                tokens_count,
                &source_key,
                &target_key
            );

            let receipt_name = self.receipt_name.get_or_default();
            let receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // let receipt_address = Key::dictionary(page_uref, owned_tokens_item_key.as_bytes());
            // TODO: should not return `source_key`
            return (receipt_string, source_key);
        }
        todo!()
    }

    /// Approves another token holder (an approved account) to transfer tokens. It
    /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    /// been burnt, or if caller tries to approve themselves as an approved account.
    pub fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
        // revert.
        self.ensure_minter_or_assigned();

        let caller = self.env().caller();
        let token_identifier = self.checked_token_identifier(token_id, token_hash);

        let owner = self.owner_of_by_id(&token_identifier);

        // Revert if caller is not token owner nor operator.
        // Only the token owner or an operator can approve an account
        let is_owner = caller == owner;
        let is_operator = !is_owner && self.operators.get_or_default(&(owner, caller));

        if !is_owner && !is_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }

        // We assume a burnt token cannot be approved
        self.ensure_not_burned(&token_identifier);

        // If token owner or operator tries to approve itself that's probably a mistake and we revert.
        self.ensure_not_caller(spender);
        self.approved
            .set(&token_identifier.to_string(), Some(spender));
        self.emit_ces_event(Approval::new(owner, spender, token_identifier));
    }

    /// Revokes an approved account to transfer tokens. It reverts
    /// if token_id is invalid, if caller is not the owner, if token has already
    /// been burnt, if caller tries to approve itself.
    pub fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        let env = self.env();
        // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
        // revert.
        self.ensure_minter_or_assigned();

        let caller = env.caller();
        let token_identifier = self.checked_token_identifier(token_id, token_hash);

        // Revert if caller is not the token owner or an operator. Only the token owner / operators can
        // revoke an approved account
        let owner = self.owner_of_by_id(&token_identifier);
        let is_owner = caller == owner;
        let is_operator = !is_owner && self.operators.get_or_default(&(owner, caller));

        if !is_owner && !is_operator {
            env.revert(CEP78Error::InvalidTokenOwner);
        }

        // We assume a burnt token cannot be revoked
        self.ensure_not_burned(&token_identifier);
        self.approved
            .set(&token_identifier.to_string(), Option::<Address>::None);
        // Emit ApprovalRevoked event.
        self.emit_ces_event(ApprovalRevoked::new(owner, token_identifier));
    }

    /// Approves all tokens owned by the caller and future to another token holder
    /// (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
    /// owner, if caller tries to approve itself as an operator.
    pub fn set_approval_for_all(&mut self, approve_all: bool, operator: Address) {
        let env = self.env();
        // If we are in minter or assigned mode it makes no sense to approve an operator. Hence we revert.
        self.ensure_minter_or_assigned();
        // If caller tries to approve itself as operator that's probably a mistake and we revert.
        self.ensure_not_caller(operator);

        let caller = env.caller();
        // Depending on approve_all we either approve all or disapprove all.
        self.operators.set(&(caller, operator), approve_all);

        let events_mode: EventsMode = self.settings.events_mode();
        if let EventsMode::CES = events_mode {
            if approve_all {
                env.emit_event(ApprovalForAll::new(caller, operator));
            } else {
                env.emit_event(RevokedForAll::new(caller, operator));
            }
        }
    }

    /// Returns if an account is operator for a token owner
    pub fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool {
        self.operators.get_or_default(&(token_owner, operator))
    }

    /// Returns the token owner given a token_id. It reverts if token_id
    /// is invalid. A burnt token still has an associated owner.
    pub fn owner_of(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.owner_of_by_id(&token_identifier)
    }

    /// Returns the approved account (if any) associated with the provided token_id
    /// Reverts if token has been burnt.
    pub fn get_approved(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>
    ) -> Option<Address> {
        let token_identifier: TokenIdentifier = self.checked_token_identifier(token_id, token_hash);

        self.ensure_not_burned(&token_identifier);
        self.approved.get(&token_identifier.to_string()).flatten()
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
        self.ensure_owner_not_caller(&token_identifier);
        self.metadata
            .update_or_revert(&updated_token_metadata, &token_identifier);

        self.emit_ces_event(MetadataUpdated::new(
            token_identifier,
            updated_token_metadata
        ));
    }

    /// Returns number of owned tokens associated with the provided token holder
    pub fn balance_of(&mut self, token_owner: Address) -> u64 {
        self.token_count.get(&token_owner).unwrap_or_default()
    }

    /// This entrypoint will upgrade the contract from the 1_0 version to the
    /// 1_1 version. The contract will insert any addition dictionaries and
    /// sentinel values that were absent in the previous version of the contract.
    /// It will also perform the necessary data transformations of historical
    /// data if needed
    pub fn migrate(&mut self, nft_package_key: String) {
        // no-op
    }

    /// This entrypoint will allow NFT owners to update their receipts from
    /// the previous owned_tokens list model to the current pagination model
    /// scheme. Calling the entrypoint will return a list of receipt names
    /// alongside the dictionary addressed to the relevant pages.
    pub fn updated_receipts(&mut self) -> Vec<(String, Address)> {
        vec![]
    }

    /// This entrypoint allows users to register with a give CEP-78 instance,
    /// allocating the necessary page table to enable the reverse lookup
    /// functionality and allowing users to pay the upfront cost of allocation
    /// resulting in more stable gas costs when minting and transferring
    /// Note: This entrypoint MUST be invoked if the reverse lookup is enabled
    /// in order to own NFTs.
    pub fn register_owner(&mut self, token_owner: Maybe<Address>) -> (String, URef) {
        if vec![
            OwnerReverseLookupMode::Complete,
            OwnerReverseLookupMode::TransfersOnly,
        ]
        .contains(&self.reverse_lookup.get_mode())
        {
            let owner = match self.ownership_mode() {
                OwnershipMode::Minter => self.env().caller(),
                OwnershipMode::Assigned | OwnershipMode::Transferable => {
                    token_owner.unwrap(&self.env())
                }
            };

            self.pagination.register_owner(&owner);
        }
        todo!()
    }

    pub fn is_whitelisted(&self, address: &Address) -> bool {
        self.whitelist.is_whitelisted(address)
    }

    pub fn get_whitelist_mode(&self) -> WhitelistMode {
        self.whitelist.get_mode()
    }

    pub fn get_collection_name(&self) -> String {
        self.info.collection_name()
    }

    pub fn is_minting_allowed(&self) -> bool {
        self.settings.allow_minting()
    }
    pub fn is_operator_burn_mode(&self) -> bool {
        self.settings.operator_burn_mode()
    }
}

impl CEP78 {
    #[inline]
    fn is_minter_or_assigned(&self) -> bool {
        matches!(
            self.ownership_mode(),
            OwnershipMode::Minter | OwnershipMode::Assigned
        )
    }

    #[inline]
    fn ensure_minter_or_assigned(&self) {
        if self.is_minter_or_assigned() {
            self.env().revert(CEP78Error::InvalidOwnershipMode)
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
        let env = self.env();
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        let token_identifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&env))
        };

        let number_of_minted_tokens = self.info.number_of_minted_tokens();
        if let NFTIdentifierMode::Ordinal = identifier_mode {
            // Revert if token_id is out of bounds
            if token_identifier.get_index().unwrap_or_revert(&env) >= number_of_minted_tokens {
                env.revert(CEP78Error::InvalidTokenIdentifier);
            }
        }
        token_identifier
    }

    #[inline]
    fn owner_of_by_id(&self, id: &TokenIdentifier) -> Address {
        match self.owners.get(&id.to_string()) {
            Some(token_owner) => token_owner,
            None => self
                .env()
                .revert(CEP78Error::MissingOwnerTokenIdentifierKey)
        }
    }

    #[inline]
    fn is_token_burned(&self, token_identifier: &TokenIdentifier) -> bool {
        self.burnt_tokens
            .get(&token_identifier.to_string())
            .is_some()
    }

    #[inline]
    fn ensure_not_owner(&self, token_identifier: &TokenIdentifier, address: &Address) {
        let owner = self.owner_of_by_id(token_identifier);
        if address == &owner {
            self.env().revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn ensure_owner_not_caller(&self, token_identifier: &TokenIdentifier) {
        let owner = self.owner_of_by_id(token_identifier);
        if self.env().caller() == owner {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }
    }

    #[inline]
    fn ensure_not_burned(&self, token_identifier: &TokenIdentifier) {
        if self.is_token_burned(token_identifier) {
            self.env().revert(CEP78Error::PreviouslyBurntToken);
        }
    }

    #[inline]
    fn ensure_not_caller(&self, address: Address) {
        if self.env().caller() == address {
            self.env().revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn ensure_caller(&self, address: Address) {
        if self.env().caller() != address {
            self.env().revert(CEP78Error::InvalidAccount);
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
            self.env().revert(CEP78Error::InvalidBurnMode)
        }
    }

    #[inline]
    fn ownership_mode(&self) -> OwnershipMode {
        self.settings.ownership_mode()
    }

    #[inline]
    fn verified_caller(&self) -> Address {
        let holder_mode = self.settings.holder_mode();
        let caller = self.env().caller();

        match (caller, holder_mode) {
            (Address::Account(_), NFTHolderMode::Contracts)
            | (Address::Contract(_), NFTHolderMode::Accounts) => {
                self.env().revert(CEP78Error::InvalidHolderMode);
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
