use odra::{args::Maybe, prelude::*, Mapping, UnwrapOrRevert, Var};
use serde::{Deserialize, Serialize};

use super::{
    constants::{METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW},
    error::CEP78Error,
    modalities::{
        MetadataMutability, MetadataRequirement, NFTIdentifierMode, NFTMetadataKind, Requirement,
        TokenIdentifier
    }
};

#[odra::module]
pub struct Metadata {
    requirements: Var<MetadataRequirement>,
    identifier_mode: Var<NFTIdentifierMode>,
    mutability: Var<MetadataMutability>,
    json_schema: Var<String>,
    validated_metadata: Mapping<(String, String), String>
}

impl Metadata {
    pub fn init(
        &mut self,
        base_metadata_kind: NFTMetadataKind,
        additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
        optional_metadata: Maybe<Vec<NFTMetadataKind>>,
        metadata_mutability: MetadataMutability,
        identifier_mode: NFTIdentifierMode,
        json_schema: Maybe<String>
    ) {
        let mut requirements = MetadataRequirement::new();
        for optional in optional_metadata.unwrap_or_default() {
            requirements.insert(optional, Requirement::Optional);
        }
        for required in additional_required_metadata.unwrap_or_default() {
            requirements.insert(required, Requirement::Required);
        }
        requirements.insert(base_metadata_kind, Requirement::Required);

        self.requirements.set(requirements);
        self.identifier_mode.set(identifier_mode);
        self.mutability.set(metadata_mutability);
        self.json_schema.set(json_schema.unwrap_or_default());
    }

    pub fn get_requirements(&self) -> MetadataRequirement {
        self.requirements.get_or_default()
    }

    pub fn get_identifier_mode(&self) -> NFTIdentifierMode {
        self.identifier_mode
            .get_or_revert_with(CEP78Error::InvalidIdentifierMode)
    }

    pub fn get_or_revert(&self, token_identifier: &TokenIdentifier) -> String {
        let env = self.env();
        let metadata_kind_list = self.get_requirements();

        for (metadata_kind, required) in metadata_kind_list {
            match required {
                Requirement::Required => {
                    let id = token_identifier.to_string();
                    let kind = get_metadata_key(&metadata_kind);
                    let metadata = self
                        .validated_metadata
                        .get(&(kind, id))
                        .unwrap_or_revert_with(&env, CEP78Error::InvalidTokenIdentifier);
                    return metadata;
                }
                _ => continue
            }
        }
        env.revert(CEP78Error::MissingTokenMetaData)
    }

    // test only
    pub fn get_metadata_by_kind(&self, token_identifier: String, kind: &NFTMetadataKind) -> String {
        let kind = get_metadata_key(kind);
        self.validated_metadata
            .get(&(kind, token_identifier))
            .unwrap_or_default()
    }

    pub fn ensure_mutability(&self, error: CEP78Error) {
        let current_mutability = self
            .mutability
            .get_or_revert_with(CEP78Error::InvalidMetadataMutability);
        if current_mutability != MetadataMutability::Mutable {
            self.env().revert(error)
        }
    }

    pub fn update_or_revert(&mut self, token_metadata: &str, token_id: &String) {
        let requirements = self.get_requirements();
        for (metadata_kind, required) in requirements {
            if required == Requirement::Unneeded {
                continue;
            }
            let token_metadata_validation = self.validate(&metadata_kind, token_metadata);
            match token_metadata_validation {
                Ok(validated_token_metadata) => {
                    let kind = get_metadata_key(&metadata_kind);
                    self.validated_metadata
                        .set(&(kind, token_id.to_owned()), validated_token_metadata);
                }
                Err(err) => {
                    self.env().revert(err);
                }
            }
        }
    }

    fn validate(&self, kind: &NFTMetadataKind, metadata: &str) -> Result<String, CEP78Error> {
        let token_schema = self.get_metadata_schema(&kind);
        match kind {
            NFTMetadataKind::CEP78 => {
                let metadata = serde_json_wasm::from_str::<MetadataCEP78>(&metadata)
                    .map_err(|_| CEP78Error::FailedToParseCep78Metadata)?;

                if let Some(name_property) = token_schema.properties.get("name") {
                    if name_property.required && metadata.name.is_empty() {
                        self.env().revert(CEP78Error::InvalidCEP78Metadata)
                    }
                }
                if let Some(token_uri_property) = token_schema.properties.get("token_uri") {
                    if token_uri_property.required && metadata.token_uri.is_empty() {
                        self.env().revert(CEP78Error::InvalidCEP78Metadata)
                    }
                }
                if let Some(checksum_property) = token_schema.properties.get("checksum") {
                    if checksum_property.required && metadata.checksum.is_empty() {
                        self.env().revert(CEP78Error::InvalidCEP78Metadata)
                    }
                }
                serde_json::to_string_pretty(&metadata)
                    .map_err(|_| CEP78Error::FailedToJsonifyCEP78Metadata)
            }
            NFTMetadataKind::NFT721 => {
                let metadata = serde_json_wasm::from_str::<MetadataNFT721>(&metadata)
                    .map_err(|_| CEP78Error::FailedToParse721Metadata)?;

                if let Some(name_property) = token_schema.properties.get("name") {
                    if name_property.required && metadata.name.is_empty() {
                        self.env().revert(CEP78Error::InvalidNFT721Metadata)
                    }
                }
                if let Some(token_uri_property) = token_schema.properties.get("token_uri") {
                    if token_uri_property.required && metadata.token_uri.is_empty() {
                        self.env().revert(CEP78Error::InvalidNFT721Metadata)
                    }
                }
                if let Some(symbol_property) = token_schema.properties.get("symbol") {
                    if symbol_property.required && metadata.symbol.is_empty() {
                        self.env().revert(CEP78Error::InvalidNFT721Metadata)
                    }
                }
                serde_json::to_string_pretty(&metadata)
                    .map_err(|_| CEP78Error::FailedToJsonifyNFT721Metadata)
            }
            NFTMetadataKind::Raw => Ok(metadata.to_owned()),
            NFTMetadataKind::CustomValidated => {
                let custom_metadata =
                    serde_json_wasm::from_str::<BTreeMap<String, String>>(&metadata)
                        .map(|attributes| CustomMetadata { attributes })
                        .map_err(|_| CEP78Error::FailedToParseCustomMetadata)?;

                for (property_name, property_type) in token_schema.properties.iter() {
                    if property_type.required
                        && custom_metadata.attributes.get(property_name).is_none()
                    {
                        self.env().revert(CEP78Error::InvalidCustomMetadata)
                    }
                }
                serde_json::to_string_pretty(&custom_metadata.attributes)
                    .map_err(|_| CEP78Error::FailedToJsonifyCustomMetadata)
            }
        }
    }

    fn get_metadata_schema(&self, kind: &NFTMetadataKind) -> CustomMetadataSchema {
        match kind {
            NFTMetadataKind::Raw => CustomMetadataSchema {
                properties: BTreeMap::new()
            },
            NFTMetadataKind::NFT721 => {
                let mut properties = BTreeMap::new();
                properties.insert(
                    "name".to_string(),
                    MetadataSchemaProperty {
                        name: "name".to_string(),
                        description: "The name of the NFT".to_string(),
                        required: true
                    }
                );
                properties.insert(
                    "symbol".to_string(),
                    MetadataSchemaProperty {
                        name: "symbol".to_string(),
                        description: "The symbol of the NFT collection".to_string(),
                        required: true
                    }
                );
                properties.insert(
                    "token_uri".to_string(),
                    MetadataSchemaProperty {
                        name: "token_uri".to_string(),
                        description: "The URI pointing to an off chain resource".to_string(),
                        required: true
                    }
                );
                CustomMetadataSchema { properties }
            }
            NFTMetadataKind::CEP78 => {
                let mut properties = BTreeMap::new();
                properties.insert(
                    "name".to_string(),
                    MetadataSchemaProperty {
                        name: "name".to_string(),
                        description: "The name of the NFT".to_string(),
                        required: true
                    }
                );
                properties.insert(
                    "token_uri".to_string(),
                    MetadataSchemaProperty {
                        name: "token_uri".to_string(),
                        description: "The URI pointing to an off chain resource".to_string(),
                        required: true
                    }
                );
                properties.insert(
                    "checksum".to_string(),
                    MetadataSchemaProperty {
                        name: "checksum".to_string(),
                        description: "A SHA256 hash of the content at the token_uri".to_string(),
                        required: true
                    }
                );
                CustomMetadataSchema { properties }
            }
            NFTMetadataKind::CustomValidated => serde_json_wasm::from_str::<CustomMetadataSchema>(
                &self.json_schema.get_or_default()
            )
            .map_err(|_| CEP78Error::InvalidJsonSchema)
            .unwrap_or_revert(&self.env())
        }
    }
}

#[derive(Serialize, Deserialize)]
#[odra::odra_type]
pub(crate) struct MetadataSchemaProperty {
    pub name: String,
    pub description: String,
    pub required: bool
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct CustomMetadataSchema {
    pub properties: BTreeMap<String, MetadataSchemaProperty>
}

// Using a structure for the purposes of serialization formatting.
#[derive(Serialize, Deserialize)]
pub(crate) struct MetadataNFT721 {
    name: String,
    symbol: String,
    token_uri: String
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MetadataCEP78 {
    name: String,
    token_uri: String,
    checksum: String
}

// Using a structure for the purposes of serialization formatting.
#[derive(Serialize, Deserialize)]
pub(crate) struct CustomMetadata {
    attributes: BTreeMap<String, String>
}

pub(crate) fn get_metadata_key(metadata_kind: &NFTMetadataKind) -> String {
    match metadata_kind {
        NFTMetadataKind::CEP78 => METADATA_CEP78,
        NFTMetadataKind::NFT721 => METADATA_NFT721,
        NFTMetadataKind::Raw => METADATA_RAW,
        NFTMetadataKind::CustomValidated => METADATA_CUSTOM_VALIDATED
    }
    .to_string()
}
