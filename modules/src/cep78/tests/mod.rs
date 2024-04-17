use self::utils::InitArgsBuilder;

use super::modalities::NFTMetadataKind;

mod acl;
mod set_variables;
mod utils;

pub(super) const COLLECTION_NAME: &str = "CEP78-Test-Collection";
pub(super) const COLLECTION_SYMBOL: &str = "CEP78";

pub(super) fn default_args_builder() -> InitArgsBuilder {
    InitArgsBuilder::default()
        .collection_name(COLLECTION_NAME.to_string())
        .collection_symbol(COLLECTION_SYMBOL.to_string())
        .total_token_supply(100u64)
        .nft_metadata_kind(NFTMetadataKind::NFT721)
}
