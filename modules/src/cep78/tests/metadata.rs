use odra::{
    args::Maybe,
    casper_types::bytesrepr::ToBytes,
    host::{Deployer, HostRef, NoArgs}
};

use crate::cep78::{
    error::CEP78Error,
    events::MetadataUpdated,
    modalities::{
        EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnershipMode, TokenIdentifier, WhitelistMode
    },
    tests::{
        utils::{
            MALFORMED_META_DATA, TEST_PRETTY_CEP78_METADATA,
            TEST_PRETTY_UPDATED_CEP78_METADATA
        },
        TEST_CUSTOM_METADATA, TEST_CUSTOM_METADATA_SCHEMA, TEST_CUSTOM_UPDATED_METADATA,
        TOKEN_HASH
    },
    token::Cep78HostRef,
    utils::MockContractHostRef
};

use super::{
    default_args_builder,
    utils::{self, TEST_PRETTY_721_META_DATA, TEST_PRETTY_UPDATED_721_META_DATA}
};

#[test]
fn should_prevent_update_in_immutable_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::NFT721)
        .identifier_mode(NFTIdentifierMode::Hash)
        .metadata_mutability(MetadataMutability::Immutable)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    contract.mint(
        env.get_account(0),
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA.to_bytes().unwrap());
    let token_hash = base16::encode_lower(&blake2b_hash);

    assert_eq!(
        contract.try_set_token_metadata(
            Maybe::None,
            Maybe::Some(token_hash),
            TEST_PRETTY_UPDATED_721_META_DATA.to_string()
        ),
        Err(CEP78Error::ForbiddenMetadataUpdate.into())
    );
}

#[test]
fn should_prevent_install_with_hash_identifier_in_mutable_mode() {
    // let env = odra_test::env();
    // let args = default_args_builder()
    //     .nft_metadata_kind(NFTMetadataKind::NFT721)
    //     .identifier_mode(NFTIdentifierMode::Hash)
    //     .metadata_mutability(MetadataMutability::Mutable)
    //     .build();
    // let _contract = Cep78HostRef::deploy(&env, args);
    // Should be possible to verify errors at installation time
    // assert_eq!(Cep78HostRef::deploy(&env, args), Err(CEP78Error::InvalidMetadataMutability));
}

#[test]
fn should_prevent_update_for_invalid_metadata() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::NFT721)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .metadata_mutability(MetadataMutability::Mutable)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    contract.mint(
        env.get_account(0),
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let original_metadata =
        contract.get_metadata_by_kind(NFTMetadataKind::NFT721, Maybe::Some(0u64), Maybe::None);
    assert_eq!(TEST_PRETTY_721_META_DATA, original_metadata);

    assert_eq!(
        contract.try_set_token_metadata(
            Maybe::Some(0u64),
            Maybe::None,
            MALFORMED_META_DATA.to_string()
        ),
        Err(CEP78Error::FailedToParse721Metadata.into())
    );
}

#[test]
fn should_prevent_metadata_update_by_non_owner_key() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::NFT721)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .metadata_mutability(MetadataMutability::Mutable)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    let token_owner = *contract.address();
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let original_metadata =
        contract.get_metadata_by_kind(NFTMetadataKind::NFT721, Maybe::Some(0u64), Maybe::None);
    assert_eq!(TEST_PRETTY_721_META_DATA, original_metadata);

    let actual_token_owner = contract.owner_of(Maybe::Some(0u64), Maybe::None);
    assert_eq!(actual_token_owner, token_owner);

    assert_eq!(
        contract.try_set_token_metadata(
            Maybe::Some(0u64),
            Maybe::None,
            TEST_PRETTY_UPDATED_721_META_DATA.to_string()
        ),
        Err(CEP78Error::InvalidTokenOwner.into())
    );
}

fn should_allow_update_for_valid_metadata_based_on_kind(
    nft_metadata_kind: NFTMetadataKind,
    identifier_mode: NFTIdentifierMode
) {
    let env = odra_test::env();
    let json_schema =
        serde_json::to_string(&*TEST_CUSTOM_METADATA_SCHEMA).expect("must convert to json schema");
    let args = default_args_builder()
        .nft_metadata_kind(nft_metadata_kind.clone())
        .identifier_mode(identifier_mode)
        .metadata_mutability(MetadataMutability::Mutable)
        .ownership_mode(OwnershipMode::Transferable)
        .json_schema(json_schema)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    let token_owner = env.get_account(0);

    let custom_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA)
        .expect("must convert to json metadata");

    let original_metadata = match &nft_metadata_kind {
        NFTMetadataKind::CEP78 => TEST_PRETTY_CEP78_METADATA,
        NFTMetadataKind::NFT721 => TEST_PRETTY_721_META_DATA,
        NFTMetadataKind::Raw => "",
        NFTMetadataKind::CustomValidated => &custom_metadata
    };

    contract.mint(token_owner, original_metadata.to_string(), Maybe::None);

    let blake2b_hash = utils::create_blake2b_hash(original_metadata.to_bytes().unwrap());
    let token_hash = base16::encode_lower(&blake2b_hash);
    let token_id = 0u64;

    let actual_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => contract.get_metadata_by_kind(
            nft_metadata_kind.clone(),
            Maybe::Some(token_id),
            Maybe::None
        ),
        NFTIdentifierMode::Hash => contract.get_metadata_by_kind(
            nft_metadata_kind.clone(),
            Maybe::None,
            Maybe::Some(token_hash.clone())
        )
    };

    assert_eq!(actual_metadata, original_metadata.to_string());

    let custom_updated_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_UPDATED_METADATA)
        .expect("must convert to json metadata");

    let updated_metadata = match &nft_metadata_kind {
        NFTMetadataKind::CEP78 => TEST_PRETTY_UPDATED_CEP78_METADATA,
        NFTMetadataKind::NFT721 => TEST_PRETTY_UPDATED_721_META_DATA,
        NFTMetadataKind::Raw => "",
        NFTMetadataKind::CustomValidated => &custom_updated_metadata
    };

    let update_result = match identifier_mode {
        NFTIdentifierMode::Ordinal => contract.try_set_token_metadata(
            Maybe::Some(token_id),
            Maybe::None,
            updated_metadata.to_string()
        ),
        NFTIdentifierMode::Hash => contract.try_set_token_metadata(
            Maybe::None,
            Maybe::Some(token_hash),
            updated_metadata.to_string()
        )
    };
    assert!(update_result.is_ok(), "failed to update metadata");

    let blake2b_hash = utils::create_blake2b_hash(updated_metadata.to_bytes().unwrap());
    let token_hash = base16::encode_lower(&blake2b_hash);

    let actual_updated_metadata = match identifier_mode {
        NFTIdentifierMode::Ordinal => {
            contract.get_metadata_by_kind(nft_metadata_kind, Maybe::Some(token_id), Maybe::None)
        }
        NFTIdentifierMode::Hash => contract.get_metadata_by_kind(
            nft_metadata_kind,
            Maybe::None,
            Maybe::Some(token_hash.clone())
        )
    };

    assert_eq!(actual_updated_metadata, updated_metadata.to_string());

    // Expect MetadataUpdated event.
    let token_id = match identifier_mode {
        NFTIdentifierMode::Ordinal => TokenIdentifier::Index(0),
        NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash)
    }
    .to_string();
    let expected_event = MetadataUpdated::new(token_id, updated_metadata.to_string());
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn should_update_metadata_for_nft721_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::NFT721,
        NFTIdentifierMode::Ordinal
    )
}

#[test]
fn should_update_metadata_for_cep78_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::CEP78,
        NFTIdentifierMode::Ordinal
    )
}

#[test]
fn should_update_metadata_for_custom_validated_using_token_id() {
    should_allow_update_for_valid_metadata_based_on_kind(
        NFTMetadataKind::CustomValidated,
        NFTIdentifierMode::Ordinal
    )
}

#[test]
fn should_get_metadata_using_token_id() {
    let env = odra_test::env();
    let mut minting_contract = MockContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_id = 0u64;

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), false);

    let minted_metadata = contract.metadata(Maybe::Some(token_id), Maybe::None);
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);
}

#[test]
fn should_get_metadata_using_token_metadata_hash() {
    let env = odra_test::env();
    let mut minting_contract = MockContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Immutable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), false);

    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA.to_bytes().unwrap());
    let token_hash = base16::encode_lower(&blake2b_hash);

    let minted_metadata = contract.metadata(Maybe::None, Maybe::Some(token_hash));
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);
}

#[test]
fn should_revert_minting_token_metadata_hash_twice() {
    let env = odra_test::env();
    let mut minting_contract = MockContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Immutable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    minting_contract.set_address(contract.address());
    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );
    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), false);

    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA.to_bytes().unwrap());
    let token_hash = base16::encode_lower(&blake2b_hash);

    let minted_metadata = contract.metadata(Maybe::None, Maybe::Some(token_hash));
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);

    assert_eq!(
        minting_contract.try_mint(TEST_PRETTY_721_META_DATA.to_string(), false),
        Err(CEP78Error::DuplicateIdentifier.into())
    );
}

#[test]
fn should_get_metadata_using_custom_token_hash() {
    let env = odra_test::env();
    let mut minting_contract = MockContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Immutable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );
    minting_contract.mint_with_hash(
        TEST_PRETTY_721_META_DATA.to_string(),
        TOKEN_HASH.to_string()
    );

    let minted_metadata: String =
        contract.metadata(Maybe::None, Maybe::Some(TOKEN_HASH.to_string()));
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);
}

#[test]
fn should_revert_minting_custom_token_hash_identifier_twice() {
    let env = odra_test::env();
    let mut minting_contract = MockContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Immutable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );
    minting_contract.mint_with_hash(
        TEST_PRETTY_721_META_DATA.to_string(),
        TOKEN_HASH.to_string()
    );

    let minted_metadata: String =
        contract.metadata(Maybe::None, Maybe::Some(TOKEN_HASH.to_string()));
    assert_eq!(minted_metadata, TEST_PRETTY_721_META_DATA);

    assert_eq!(
        minting_contract.try_mint_with_hash(
            TEST_PRETTY_721_META_DATA.to_string(),
            TOKEN_HASH.to_string()
        ),
        Err(CEP78Error::DuplicateIdentifier.into())
    );
}

#[test]
fn should_require_valid_json_schema_when_kind_is_custom_validated() {
    let _env = odra_test::env();
    let _args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CustomValidated)
        .build();

    /*let error = builder.get_error().expect("must have error");
    support::assert_expected_error(error, 68, "valid json_schema is required")*/
}

#[test]
fn should_require_json_schema_when_kind_is_custom_validated() {
    let env = odra_test::env();
    let nft_metadata_kind = NFTMetadataKind::CustomValidated;

    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Mutable)
        .nft_metadata_kind(nft_metadata_kind)
        .json_schema("".to_string())
        .build();
    let _contract = Cep78HostRef::deploy(&env, args);

    /*let error = builder.get_error().expect("must have error");
    support::assert_expected_error(error, 67, "json_schema is required")*/
}

fn should_not_require_json_schema_when_kind_is(nft_metadata_kind: NFTMetadataKind) {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Mutable)
        .nft_metadata_kind(nft_metadata_kind.clone())
        .json_schema("".to_string())
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);

    let original_metadata = match &nft_metadata_kind {
        NFTMetadataKind::CEP78 => TEST_PRETTY_CEP78_METADATA,
        NFTMetadataKind::NFT721 => TEST_PRETTY_721_META_DATA,
        NFTMetadataKind::Raw => "",
        _ => panic!(
            "NFTMetadataKind {:?} not supported without json_schema",
            nft_metadata_kind
        )
    };

    assert!(contract
        .try_mint(
            env.get_account(0),
            original_metadata.to_string(),
            Maybe::None
        )
        .is_ok());
}

#[test]
fn should_not_require_json_schema_when_kind_is_not_custom_validated() {
    should_not_require_json_schema_when_kind_is(NFTMetadataKind::Raw);
    should_not_require_json_schema_when_kind_is(NFTMetadataKind::CEP78);
    should_not_require_json_schema_when_kind_is(NFTMetadataKind::NFT721);
}
