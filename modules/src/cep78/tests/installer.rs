use odra::host::Deployer;

use crate::cep78::{
    modalities::MintingMode,
    tests::{default_args_builder, COLLECTION_NAME, COLLECTION_SYMBOL},
    token::CEP78HostRef
};

#[test]
fn should_install_contract() {
    let env = odra_test::env();

    let args = default_args_builder()
        .collection_name(COLLECTION_NAME.to_string())
        .collection_symbol(COLLECTION_SYMBOL.to_string())
        .total_token_supply(1u64)
        .allow_minting(true)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);

    assert_eq!(&contract.get_collection_name(), COLLECTION_NAME);
    assert_eq!(&contract.get_collection_symbol(), COLLECTION_SYMBOL);
    assert_eq!(contract.get_total_supply(), 1u64);
    assert_eq!(contract.is_minting_allowed(), true);
    assert_eq!(contract.get_minting_mode(), MintingMode::Installer);
    assert_eq!(contract.get_number_of_minted_tokens(), 0u64);

    // Expects Schemas to be registerd.
    // let expected_schemas = Schemas::new()
    //     .with::<Mint>()
    //     .with::<Burn>()
    //     .with::<Approval>()
    //     .with::<ApprovalRevoked>()
    //     .with::<ApprovalForAll>()
    //     .with::<Transfer>()
    //     .with::<MetadataUpdated>()
    //     .with::<VariablesSet>()
    //     .with::<Migration>();
    // let actual_schemas: Schemas = support::query_stored_value(
    //     &builder,
    //     nft_contract_key,
    //     vec![casper_event_standard::EVENTS_SCHEMA.to_string()],
    // );
    // assert_eq!(actual_schemas, expected_schemas, "Schemas mismatch.");
}

#[test]
#[ignore = "Not applicable in Odra, init is not allowed after installation by design"]
fn should_only_allow_init_during_installation_session() {}

#[test]
fn should_install_with_allow_minting_set_to_false() {
    let env = odra_test::env();

    let args = default_args_builder().allow_minting(false).build();
    let contract = CEP78HostRef::deploy(&env, args);
    assert_eq!(contract.is_minting_allowed(), false);
}

#[test]
#[ignore = "Odra interface does not allow to pass a wrong type"]
fn should_reject_invalid_collection_name() {}
#[test]
#[ignore = "Odra interface does not allow to pass a wrong type"]
fn should_reject_invalid_collection_symbol() {}

/*
#[test]
fn should_reject_non_numerical_total_token_supply_value() {
    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_invalid_total_token_supply(
                CLValue::from_t::<String>("".to_string()).expect("expected CLValue"),
            );
    support::assert_expected_invalid_installer_request(
        install_request_builder,
        26,
        "should reject installation when given an invalid total supply value",
    );
}

#[test]
fn should_install_with_contract_holder_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let contract_whitelist = vec![Key::from(ContractHash::default())];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_holder_mode(NFTHolderMode::Contracts)
        .with_whitelist_mode(WhitelistMode::Unlocked)
        .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
        .with_minting_mode(MintingMode::Acl)
        .with_acl_whitelist(contract_whitelist);

    builder
        .exec(install_request.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let actual_holder_mode: u8 = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_HOLDER_MODE.to_string()],
    );

    assert_eq!(
        actual_holder_mode,
        NFTHolderMode::Contracts as u8,
        "holder mode is not set to contracts"
    );

    let actual_whitelist_mode: u8 = support::query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_WHITELIST_MODE.to_string()],
    );

    assert_eq!(
        actual_whitelist_mode,
        WhitelistMode::Unlocked as u8,
        "whitelist mode is not set to unlocked"
    );

    let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
        &builder,
        &nft_contract_key,
        ACL_WHITELIST,
        &ContractHash::default().to_string(),
    );

    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");
}

fn should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
    nft_holder_mode: NFTHolderMode,
) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_holder_mode(nft_holder_mode)
            .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
            .with_whitelist_mode(WhitelistMode::Locked)
            .with_minting_mode(MintingMode::Acl);

    support::assert_expected_invalid_installer_request(
        install_request_builder,
        162,
        "should fail execution since whitelist mode is locked and the provided whitelist is empty",
    );
}

#[test]
fn should_disallow_installation_of_contract_with_empty_locked_whitelist() {
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Accounts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Contracts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Mixed,
    );
}

#[test]
fn should_disallow_installation_with_zero_issuance() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(0u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 123u16, "cannot install when issuance is 0");
}

#[test]
fn should_disallow_installation_with_supply_exceeding_hard_cap() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000_001u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        133u16,
        "cannot install when issuance is more than 1_000_000",
    );
}

#[test]
fn should_prevent_installation_with_ownership_and_minting_modality_conflict() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        130u16,
        "cannot install when Ownership::Minter and MintingMode::Installer",
    );
}

#[test]
fn should_prevent_installation_with_ownership_minter_and_owner_reverse_lookup_mode_transfer_only() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        140u16,
        "cannot install when Ownership::Minter and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_prevent_installation_with_ownership_assigned_and_owner_reverse_lookup_mode_transfer_only()
{
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Assigned)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_failure().commit();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        140u16,
        "cannot install when Ownership::Assigned and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_allow_installation_with_ownership_transferable_and_owner_reverse_lookup_mode_transfer_only(
) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder
        .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
        .commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1_000u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();

    builder.exec(install_request).expect_success().commit();
}
*/
