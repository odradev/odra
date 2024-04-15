use crate::cep78::{
    modalities::{
        BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTKind, NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode
    },
    token::{CEP78HostRef, CEP78InitArgs}
};
use derive_builder::Builder;
use odra::{
    args::Maybe,
    casper_types::{runtime_args, RuntimeArgs},
    Address
};

#[derive(Default, Builder)]
#[builder(setter(into))]
pub struct InitArgs {
    collection_name: String,
    collection_symbol: String,
    total_token_supply: u64,
    allow_minting: Maybe<bool>,
    minting_mode: Maybe<MintingMode>,
    ownership_mode: OwnershipMode,
    nft_kind: NFTKind,
    holder_mode: Maybe<NFTHolderMode>,
    whitelist_mode: Maybe<WhitelistMode>,
    acl_white_list: Maybe<Vec<String>>,
    acl_package_mode: Maybe<Vec<String>>,
    package_operator_mode: Maybe<Vec<String>>,
    json_schema: Maybe<String>,
    receipt_name: Maybe<String>,
    identifier_mode: u8,
    burn_mode: Maybe<u8>,
    operator_burn_mode: Maybe<bool>,
    nft_metadata_kind: NFTMetadataKind,
    metadata_mutability: MetadataMutability,
    owner_reverse_lookup_mode: Maybe<OwnerReverseLookupMode>,
    events_mode: EventsMode,
    transfer_filter_contract_contract_key: Maybe<Address>,
    additional_required_metadata: Maybe<Vec<u8>>,
    optional_metadata: Maybe<Vec<u8>>
}

impl Into<RuntimeArgs> for InitArgs {
    fn into(self) -> RuntimeArgs {
        runtime_args! {
            "collection_name" => self.collection_name,
            "collection_symbol" => self.collection_symbol,
            "total_token_supply" => self.total_token_supply,
            "allow_minting" => self.allow_minting.unwrap_or_default(),
            "minting_mode" => self.minting_mode.unwrap_or(MintingMode::Installer) as u8,
            "ownership_mode" => self.ownership_mode as u8,
            "nft_kind" => self.nft_kind as u8,
            "holder_mode" => self.holder_mode.unwrap_or(NFTHolderMode::Accounts) as u8,
            "whitelist_mode" => self.whitelist_mode.unwrap_or(WhitelistMode::Unlocked) as u8,
            "acl_white_list" => self.acl_white_list.unwrap_or_default(),
            "acl_package_mode" => self.acl_package_mode.unwrap_or_default(),
            "package_operator_mode" => self.package_operator_mode.unwrap_or_default(),
            "json_schema" => self.json_schema.unwrap_or_default(),
            "receipt_name" => self.receipt_name.unwrap_or_default(),
            "identifier_mode" => self.identifier_mode,
            "burn_mode" => self.burn_mode.unwrap_or_default(),
            "operator_burn_mode" => self.operator_burn_mode.unwrap_or_default(),
            "nft_metadata_kind" => self.nft_metadata_kind as u8,
            "metadata_mutability" => self.metadata_mutability as u8,
            "owner_reverse_lookup_mode" => self.owner_reverse_lookup_mode.unwrap_or(OwnerReverseLookupMode::NoLookUp) as u8,
            "events_mode" => self.events_mode as u8,
            // "transfer_filter_contract_contract_key" => self.transfer_filter_contract_contract_key,
            "additional_required_metadata" => self.additional_required_metadata.unwrap_or_default(),
            "optional_metadata" => self.optional_metadata.unwrap_or_default(),
        }
    }
}

impl odra::host::InitArgs for InitArgs {
    fn validate(_expected_ident: &str) -> bool {
        true
    }
}
