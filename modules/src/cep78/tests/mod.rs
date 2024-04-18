use alloc::collections::BTreeMap;
use once_cell::sync::Lazy;

use self::utils::InitArgsBuilder;

use super::{
    metadata::{CustomMetadataSchema, MetadataSchemaProperty},
    modalities::NFTMetadataKind
};

mod acl;
mod installer;
mod mint;
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

pub(super) static TEST_CUSTOM_METADATA: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
    let mut attributes = BTreeMap::new();
    attributes.insert("deity_name".to_string(), "Baldur".to_string());
    attributes.insert("mythology".to_string(), "Nordic".to_string());
    attributes
});

pub(crate) static TEST_CUSTOM_METADATA_SCHEMA: Lazy<CustomMetadataSchema> = Lazy::new(|| {
    let mut properties = BTreeMap::new();
    properties.insert(
        "deity_name".to_string(),
        MetadataSchemaProperty {
            name: "deity_name".to_string(),
            description: "The name of deity from a particular pantheon.".to_string(),
            required: true
        }
    );
    properties.insert(
        "mythology".to_string(),
        MetadataSchemaProperty {
            name: "mythology".to_string(),
            description: "The mythology the deity belongs to.".to_string(),
            required: true
        }
    );
    CustomMetadataSchema { properties }
});
