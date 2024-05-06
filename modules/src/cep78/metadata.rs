use odra::{args::Maybe, prelude::*, SubModule, UnwrapOrRevert};
use serde::{Deserialize, Serialize};

use crate::simple_storage;

use super::{
    constants::{
        IDENTIFIER_MODE, JSON_SCHEMA, METADATA_MUTABILITY, NFT_METADATA_KIND, NFT_METADATA_KINDS
    },
    constants::{METADATA_CEP78, METADATA_CUSTOM_VALIDATED, METADATA_NFT721, METADATA_RAW},
    error::CEP78Error,
    modalities::{
        MetadataMutability, MetadataRequirement, NFTIdentifierMode, NFTMetadataKind, Requirement,
        TokenIdentifier
    }
};

simple_storage!(
    Cep78MetadataRequirement,
    MetadataRequirement,
    NFT_METADATA_KINDS,
    CEP78Error::MissingNFTMetadataKind
);
simple_storage!(
    Cep78NFTMetadataKind,
    NFTMetadataKind,
    NFT_METADATA_KIND,
    CEP78Error::MissingNFTMetadataKind
);
simple_storage!(
    Cep78IdentifierMode,
    NFTIdentifierMode,
    IDENTIFIER_MODE,
    CEP78Error::MissingIdentifierMode
);
simple_storage!(
    Cep78MetadataMutability,
    MetadataMutability,
    METADATA_MUTABILITY,
    CEP78Error::MissingMetadataMutability
);
simple_storage!(
    Cep78JsonSchema,
    String,
    JSON_SCHEMA,
    CEP78Error::MissingJsonSchema
);

#[odra::module]
pub struct Cep78ValidatedMetadata;

#[odra::module]
impl Cep78ValidatedMetadata {
    pub fn set(&self, kind: &NFTMetadataKind, token_id: &String, value: String) {
        let dictionary_name = get_metadata_key(kind);
        self.env()
            .set_dictionary_value(dictionary_name, token_id.as_bytes(), value);
    }

    pub fn get(&self, kind: &NFTMetadataKind, token_id: &String) -> String {
        let dictionary_name = get_metadata_key(kind);
        let env = self.env();
        env.get_dictionary_value(dictionary_name, token_id.as_bytes())
            .unwrap_or_revert_with(&env, CEP78Error::InvalidTokenIdentifier)
    }
}

#[odra::module]
pub struct Metadata {
    requirements: SubModule<Cep78MetadataRequirement>,
    identifier_mode: SubModule<Cep78IdentifierMode>,
    mutability: SubModule<Cep78MetadataMutability>,
    json_schema: SubModule<Cep78JsonSchema>,
    validated_metadata: SubModule<Cep78ValidatedMetadata>,
    nft_metadata_kind: SubModule<Cep78NFTMetadataKind>
}

impl Metadata {
    pub fn init(
        &mut self,
        base_metadata_kind: NFTMetadataKind,
        additional_required_metadata: Maybe<Vec<NFTMetadataKind>>,
        optional_metadata: Maybe<Vec<NFTMetadataKind>>,
        metadata_mutability: MetadataMutability,
        identifier_mode: NFTIdentifierMode,
        json_schema: String
    ) {
        let mut requirements = MetadataRequirement::new();
        for optional in optional_metadata.unwrap_or_default() {
            requirements.insert(optional, Requirement::Optional);
        }
        for required in additional_required_metadata.unwrap_or_default() {
            requirements.insert(required, Requirement::Required);
        }
        requirements.insert(base_metadata_kind.clone(), Requirement::Required);

        // Attempt to parse the provided schema if the `CustomValidated` metadata kind is required or
        // optional and fail installation if the schema cannot be parsed.
        if let Some(req) = requirements.get(&NFTMetadataKind::CustomValidated) {
            if req == &Requirement::Required || req == &Requirement::Optional {
                serde_json_wasm::from_str::<CustomMetadataSchema>(&json_schema)
                    .map_err(|_| CEP78Error::InvalidJsonSchema)
                    .unwrap_or_revert(&self.env());
            }
        }
        self.nft_metadata_kind.set(base_metadata_kind);
        self.requirements.set(requirements);
        self.identifier_mode.set(identifier_mode);
        self.mutability.set(metadata_mutability);
        self.json_schema.set(json_schema);
    }

    pub fn get_requirements(&self) -> MetadataRequirement {
        self.requirements.get()
    }

    pub fn get_identifier_mode(&self) -> NFTIdentifierMode {
        self.identifier_mode.get()
    }

    pub fn get_or_revert(&self, token_identifier: &TokenIdentifier) -> String {
        let env = self.env();
        let metadata_kind_list = self.get_requirements();

        for (metadata_kind, required) in metadata_kind_list {
            match required {
                Requirement::Required => {
                    let id = token_identifier.to_string();
                    let metadata = self.validated_metadata.get(&metadata_kind, &id);
                    return metadata;
                }
                _ => continue
            }
        }
        env.revert(CEP78Error::MissingTokenMetaData)
    }

    // test only
    pub fn get_metadata_by_kind(&self, token_identifier: String, kind: &NFTMetadataKind) -> String {
        self.validated_metadata.get(kind, &token_identifier)
    }

    pub fn ensure_mutability(&self, error: CEP78Error) {
        let current_mutability = self.mutability.get();
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
                    self.validated_metadata
                        .set(&metadata_kind, token_id, validated_token_metadata);
                }
                Err(err) => {
                    self.env().revert(err);
                }
            }
        }
    }

    fn validate(&self, kind: &NFTMetadataKind, metadata: &str) -> Result<String, CEP78Error> {
        let token_schema = self.get_metadata_schema(kind);
        match kind {
            NFTMetadataKind::CEP78 => {
                let metadata = serde_json_wasm::from_str::<MetadataCEP78>(metadata)
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
                let metadata = serde_json_wasm::from_str::<MetadataNFT721>(metadata)
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
                    serde_json_wasm::from_str::<BTreeMap<String, String>>(metadata)
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
            NFTMetadataKind::CustomValidated => {
                serde_json_wasm::from_str::<CustomMetadataSchema>(&self.json_schema.get())
                    .map_err(|_| CEP78Error::InvalidJsonSchema)
                    .unwrap_or_revert(&self.env())
            }
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
