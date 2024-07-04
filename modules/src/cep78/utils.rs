#![allow(dead_code)]
use super::reverse_lookup::PAGE_SIZE;

#[cfg(not(target_arch = "wasm32"))]
use crate::cep78::{
    modalities::{
        BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTKind, NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode
    },
    token::TestCep78InitArgs
};
use odra::{args::Maybe, prelude::*, Address, External, Var};

pub fn address_to_key(address: &Address) -> String {
    match address {
        Address::Account(account) => account.to_string(),
        Address::Contract(contract) => contract.to_string()
    }
}

pub fn max_number_of_pages(total_token_supply: u64) -> u64 {
    if total_token_supply < PAGE_SIZE {
        1
    } else {
        let max_number_of_pages = total_token_supply / PAGE_SIZE;
        let overflow = total_token_supply % PAGE_SIZE;
        if overflow == 0 {
            max_number_of_pages
        } else {
            max_number_of_pages + 1
        }
    }
}

#[odra::module]
struct MockDummyContract;

#[odra::module]
impl MockDummyContract {}

#[odra::module]
pub struct MockCep78TransferFilter {
    value: Var<u8>
}

#[odra::module]
impl MockCep78TransferFilter {
    pub fn set_return_value(&mut self, return_value: u8) {
        self.value.set(return_value);
    }

    pub fn can_transfer(&self) -> u8 {
        self.value.get_or_default()
    }
}

#[odra::module]
struct MockCep78Operator {
    nft_contract: External<NftContractContractRef>
}

#[odra::module]
impl MockCep78Operator {
    pub fn set_address(&mut self, nft_contract: &Address) {
        self.nft_contract.set(*nft_contract);
    }

    pub fn mint(&mut self, token_metadata: String, is_reverse_lookup_enabled: bool) {
        let addr = self.env().self_address();
        if is_reverse_lookup_enabled {
            self.nft_contract.register_owner(Maybe::Some(addr));
        }

        self.nft_contract.mint(addr, token_metadata, Maybe::None)
    }

    pub fn mint_with_hash(&mut self, token_metadata: String, token_hash: String) {
        let addr = self.env().self_address();
        self.nft_contract
            .mint(addr, token_metadata, Maybe::Some(token_hash))
    }

    pub fn burn(&mut self, token_id: u64) {
        self.nft_contract.burn(Maybe::Some(token_id), Maybe::None);
    }

    pub fn mint_for(&mut self, token_owner: Address, token_metadata: String) {
        self.nft_contract
            .mint(token_owner, token_metadata, Maybe::None)
    }

    pub fn transfer(&mut self, token_id: u64, target: Address) {
        let address = self.env().self_address();
        self.nft_contract
            .transfer(Maybe::Some(token_id), Maybe::None, address, target)
    }
    pub fn transfer_from(&mut self, token_id: u64, source: Address, target: Address) {
        self.nft_contract
            .transfer(Maybe::Some(token_id), Maybe::None, source, target)
    }

    pub fn approve(&mut self, spender: Address, token_id: u64) {
        self.nft_contract
            .approve(spender, Maybe::Some(token_id), Maybe::None)
    }

    pub fn revoke(&mut self, token_id: u64) {
        self.nft_contract.revoke(Maybe::Some(token_id), Maybe::None)
    }
}

#[odra::external_contract]
trait NftContract {
    fn mint(&mut self, token_owner: Address, token_meta_data: String, token_hash: Maybe<String>);
    fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
    fn register_owner(&mut self, token_owner: Maybe<Address>) -> String;
    fn transfer(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        source_key: Address,
        target_key: Address
    );
    fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>);
    fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct InitArgsBuilder {
    collection_name: String,
    collection_symbol: String,
    total_token_supply: u64,
    allow_minting: Maybe<bool>,
    minting_mode: Maybe<MintingMode>,
    ownership_mode: OwnershipMode,
    nft_kind: NFTKind,
    receipt_name: String,
    holder_mode: Maybe<NFTHolderMode>,
    whitelist_mode: Maybe<WhitelistMode>,
    acl_white_list: Maybe<Vec<Address>>,
    json_schema: Maybe<String>,
    identifier_mode: NFTIdentifierMode,
    burn_mode: Maybe<BurnMode>,
    operator_burn_mode: Maybe<bool>,
    nft_metadata_kind: NFTMetadataKind,
    metadata_mutability: MetadataMutability,
    owner_reverse_lookup_mode: Maybe<OwnerReverseLookupMode>,
    events_mode: Maybe<EventsMode>,
    transfer_filter_contract_contract_key: Maybe<Address>,
    additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
    optional_metadata: Maybe<Vec<NFTMetadataKind>>
}

#[cfg(not(target_arch = "wasm32"))]
impl InitArgsBuilder {
    pub fn collection_name(mut self, collection_name: String) -> Self {
        self.collection_name = collection_name;
        self
    }

    pub fn collection_symbol(mut self, collection_symbol: String) -> Self {
        self.collection_symbol = collection_symbol;
        self
    }

    pub fn total_token_supply(mut self, total_token_supply: u64) -> Self {
        self.total_token_supply = total_token_supply;
        self
    }

    pub fn allow_minting(mut self, allow_minting: bool) -> Self {
        self.allow_minting = Maybe::Some(allow_minting);
        self
    }

    pub fn nft_kind(mut self, nft_kind: NFTKind) -> Self {
        self.nft_kind = nft_kind;
        self
    }

    pub fn minting_mode(mut self, minting_mode: MintingMode) -> Self {
        self.minting_mode = Maybe::Some(minting_mode);
        self
    }

    pub fn ownership_mode(mut self, ownership_mode: OwnershipMode) -> Self {
        self.ownership_mode = ownership_mode;
        self
    }

    pub fn holder_mode(mut self, holder_mode: NFTHolderMode) -> Self {
        self.holder_mode = Maybe::Some(holder_mode);
        self
    }

    pub fn whitelist_mode(mut self, whitelist_mode: WhitelistMode) -> Self {
        self.whitelist_mode = Maybe::Some(whitelist_mode);
        self
    }

    pub fn acl_white_list(mut self, acl_white_list: Vec<Address>) -> Self {
        self.acl_white_list = Maybe::Some(acl_white_list);
        self
    }

    pub fn json_schema(mut self, json_schema: String) -> Self {
        self.json_schema = Maybe::Some(json_schema);
        self
    }

    pub fn identifier_mode(mut self, identifier_mode: NFTIdentifierMode) -> Self {
        self.identifier_mode = identifier_mode;
        self
    }

    pub fn receipt_name(mut self, receipt_name: String) -> Self {
        self.receipt_name = receipt_name;
        self
    }

    pub fn burn_mode(mut self, burn_mode: BurnMode) -> Self {
        self.burn_mode = Maybe::Some(burn_mode);
        self
    }

    pub fn operator_burn_mode(mut self, operator_burn_mode: bool) -> Self {
        self.operator_burn_mode = Maybe::Some(operator_burn_mode);
        self
    }

    pub fn nft_metadata_kind(mut self, nft_metadata_kind: NFTMetadataKind) -> Self {
        self.nft_metadata_kind = nft_metadata_kind;
        self
    }

    pub fn metadata_mutability(mut self, metadata_mutability: MetadataMutability) -> Self {
        self.metadata_mutability = metadata_mutability;
        self
    }

    pub fn owner_reverse_lookup_mode(
        mut self,
        owner_reverse_lookup_mode: OwnerReverseLookupMode
    ) -> Self {
        self.owner_reverse_lookup_mode = Maybe::Some(owner_reverse_lookup_mode);
        self
    }

    pub fn events_mode(mut self, events_mode: EventsMode) -> Self {
        self.events_mode = Maybe::Some(events_mode);
        self
    }

    pub fn transfer_filter_contract_contract_key(
        mut self,
        transfer_filter_contract_contract_key: Address
    ) -> Self {
        self.transfer_filter_contract_contract_key =
            Maybe::Some(transfer_filter_contract_contract_key);
        self
    }

    pub fn additional_required_metadata(
        mut self,
        additional_required_metadata: Vec<NFTMetadataKind>
    ) -> Self {
        self.additional_required_metadata = Maybe::Some(additional_required_metadata);
        self
    }

    pub fn optional_metadata(mut self, optional_metadata: Vec<NFTMetadataKind>) -> Self {
        self.optional_metadata = Maybe::Some(optional_metadata);
        self
    }

    pub fn build(self) -> TestCep78InitArgs {
        TestCep78InitArgs {
            collection_name: self.collection_name,
            collection_symbol: self.collection_symbol,
            total_token_supply: self.total_token_supply,
            allow_minting: self.allow_minting,
            minting_mode: self.minting_mode,
            ownership_mode: self.ownership_mode,
            nft_kind: self.nft_kind,
            holder_mode: self.holder_mode,
            whitelist_mode: self.whitelist_mode,
            acl_whitelist: self.acl_white_list,
            json_schema: self.json_schema,
            receipt_name: self.receipt_name,
            identifier_mode: self.identifier_mode,
            burn_mode: self.burn_mode,
            operator_burn_mode: self.operator_burn_mode,
            nft_metadata_kind: self.nft_metadata_kind,
            metadata_mutability: self.metadata_mutability,
            owner_reverse_lookup_mode: self.owner_reverse_lookup_mode,
            events_mode: self.events_mode,
            transfer_filter_contract_contract: self.transfer_filter_contract_contract_key,
            additional_required_metadata: self.additional_required_metadata,
            optional_metadata: self.optional_metadata
        }
    }
}
