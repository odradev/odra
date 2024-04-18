use crate::cep78::{
    modalities::{
        BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTKind, NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode
    },
    token::CEP78InitArgs
};
use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar
};
use odra::{args::Maybe, casper_types::BLAKE2B_DIGEST_LENGTH, prelude::*, Address, ContractRef};

use super::acl::NftContractContractRef;

#[derive(Default)]
pub struct InitArgsBuilder {
    collection_name: String,
    collection_symbol: String,
    total_token_supply: u64,
    allow_minting: Maybe<bool>,
    minting_mode: Maybe<MintingMode>,
    ownership_mode: OwnershipMode,
    nft_kind: NFTKind,
    holder_mode: Maybe<NFTHolderMode>,
    whitelist_mode: Maybe<WhitelistMode>,
    acl_white_list: Maybe<Vec<Address>>,
    acl_package_mode: Maybe<Vec<String>>,
    package_operator_mode: Maybe<Vec<String>>,
    json_schema: Maybe<String>,
    receipt_name: Maybe<String>,
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

    pub fn minting_mode(mut self, minting_mode: MintingMode) -> Self {
        self.minting_mode = Maybe::Some(minting_mode);
        self
    }

    pub fn ownership_mode(mut self, ownership_mode: OwnershipMode) -> Self {
        self.ownership_mode = ownership_mode;
        self
    }

    pub fn nft_kind(mut self, nft_kind: NFTKind) -> Self {
        self.nft_kind = nft_kind;
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

    pub fn acl_package_mode(mut self, acl_package_mode: Vec<String>) -> Self {
        self.acl_package_mode = Maybe::Some(acl_package_mode);
        self
    }

    pub fn package_operator_mode(mut self, package_operator_mode: Vec<String>) -> Self {
        self.package_operator_mode = Maybe::Some(package_operator_mode);
        self
    }

    pub fn json_schema(mut self, json_schema: String) -> Self {
        self.json_schema = Maybe::Some(json_schema);
        self
    }

    pub fn receipt_name(mut self, receipt_name: String) -> Self {
        self.receipt_name = Maybe::Some(receipt_name);
        self
    }

    pub fn identifier_mode(mut self, identifier_mode: NFTIdentifierMode) -> Self {
        self.identifier_mode = identifier_mode;
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

    pub fn build(self) -> CEP78InitArgs {
        CEP78InitArgs {
            collection_name: self.collection_name,
            collection_symbol: self.collection_symbol,
            total_token_supply: self.total_token_supply,
            allow_minting: self.allow_minting,
            minting_mode: self.minting_mode,
            ownership_mode: self.ownership_mode,
            nft_kind: self.nft_kind,
            holder_mode: self.holder_mode,
            whitelist_mode: self.whitelist_mode,
            acl_white_list: self.acl_white_list,
            json_schema: self.json_schema,
            receipt_name: self.receipt_name,
            nft_identifier_mode: self.identifier_mode,
            burn_mode: self.burn_mode,
            operator_burn_mode: self.operator_burn_mode,
            nft_metadata_kind: self.nft_metadata_kind,
            metadata_mutability: self.metadata_mutability,
            owner_reverse_lookup_mode: self.owner_reverse_lookup_mode,
            events_mode: self.events_mode,
            transfer_filter_contract_contract_key: self.transfer_filter_contract_contract_key,
            additional_required_metadata: self.additional_required_metadata,
            optional_metadata: self.optional_metadata
        }
    }
}

pub const TEST_PRETTY_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}"#;
pub const TEST_PRETTY_UPDATED_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.foobar.com"
}"#;
pub const TEST_PRETTY_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.barfoo.com",
  "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
pub const TEST_PRETTY_UPDATED_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.foobar.com",
  "checksum": "fda4feaa137e83972db628e521c92159f5dc253da1565c9da697b8ad845a0788"
}"#;
pub const TEST_COMPACT_META_DATA: &str =
    r#"{"name": "John Doe","symbol": "abc","token_uri": "https://www.barfoo.com"}"#;
pub const MALFORMED_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": abc,
  "token_uri": "https://www.barfoo.com"
}"#;

#[odra::module]
struct DummyContract;

#[odra::module]
impl DummyContract {}

#[odra::module]
struct TestContract;

#[odra::module]
impl TestContract {
    pub fn mint(
        &mut self,
        nft_contract_address: &Address,
        token_metadata: String
    ) -> (String, Address, String) {
        NftContractContractRef::new(self.env(), *nft_contract_address)
            .mint(self.env().self_address(), token_metadata)
    }

    pub fn mint_for(
        &mut self,
        nft_contract_address: &Address,
        token_owner: Address,
        token_metadata: String
    ) -> (String, Address, String) {
        NftContractContractRef::new(self.env(), *nft_contract_address)
            .mint(token_owner, token_metadata)
    }
}
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Write;
pub(crate) fn create_blake2b_hash<T: AsRef<[u8]>>(data: T) -> [u8; BLAKE2B_DIGEST_LENGTH] {
    let mut result = [0u8; 32];
    let mut hasher = <Blake2bVar as VariableOutput>::new(32).expect("should create hasher");
    let _ = hasher.write(data.as_ref());
    hasher
        .finalize_variable(&mut result)
        .expect("should copy hash to the result array");
    result
}
