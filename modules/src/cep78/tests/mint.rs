use odra::{
    args::Maybe,
    host::{Deployer, HostEnv, HostRef}
};
use serde::{Deserialize, Serialize};

use crate::cep78::{
    error::CEP78Error,
    events::{ApprovalForAll, Mint, RevokedForAll},
    modalities::{
        EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, TokenIdentifier, WhitelistMode
    },
    reverse_lookup::PAGE_SIZE,
    tests::{
        utils::{
            self, MALFORMED_META_DATA, TEST_COMPACT_META_DATA, TEST_PRETTY_CEP78_METADATA,
            TEST_PRETTY_UPDATED_CEP78_METADATA
        },
        TEST_CUSTOM_METADATA, TEST_CUSTOM_METADATA_SCHEMA
    },
    token::{TestCep78, TestCep78HostRef}
};

use super::{default_args_builder, utils::TEST_PRETTY_721_META_DATA};

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    name: String,
    symbol: String,
    token_uri: String
}

fn default_token() -> (TestCep78HostRef, HostEnv) {
    let env = odra_test::env();
    let args = default_args_builder()
        .total_token_supply(2u64)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    (TestCep78::deploy(&env, args), env)
}

#[test]
fn should_disallow_minting_when_allow_minting_is_set_to_false() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::NFT721)
        .allow_minting(false)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    assert_eq!(
        contract.try_mint(
            env.get_account(0),
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::MintingIsPaused.into()),
        "should now allow minting when minting is disabled",
    );
}

#[test]
#[ignore = "Odra uses proxy pattern for contract calls, so this test is not applicable"]
fn entry_points_with_ret_should_return_correct_value() {}

#[test]
fn should_mint() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    assert!(contract
        .try_mint(
            token_owner,
            TEST_PRETTY_CEP78_METADATA.to_string(),
            Maybe::None
        )
        .is_ok());

    // Expect Mint event.
    let expected_event = Mint::new(
        token_owner,
        TokenIdentifier::Index(0).to_string(),
        TEST_PRETTY_CEP78_METADATA.to_string()
    );
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn mint_should_return_dictionary_key_to_callers_owned_tokens() {
    let (mut contract, env) = default_token();

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_page = contract.get_page(0u64);
    let expected_page = {
        let mut page = vec![false; PAGE_SIZE as usize];
        page[0] = true;
        page
    };
    assert_eq!(actual_page, expected_page);
}

#[test]
fn mint_should_increment_number_of_minted_tokens_by_one_and_add_public_key_to_token_owners() {
    let (mut contract, env) = default_token();
    let owner = env.get_account(0);

    contract.register_owner(Maybe::Some(owner));
    assert!(contract
        .try_mint(owner, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None)
        .is_ok());

    assert_eq!(
        contract.get_number_of_minted_tokens(),
        1u64,
        "number_of_minted_tokens initialized at installation should have incremented by one"
    );

    let token_id = 0u64;

    let actual_token_meta_data =
        contract.get_metadata_by_kind(NFTMetadataKind::NFT721, Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, owner);

    let token_page = contract.get_page_by_token_id(token_id);
    assert!(token_page[0]);

    // If total_token_supply is initialized to 1 the following test should fail.
    // If we set total_token_supply > 1 it should pass
    assert!(contract
        .try_mint(owner, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None)
        .is_ok());
}

#[test]
fn should_set_meta_data() {
    let (mut contract, env) = default_token();

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_token_meta_data =
        contract.get_metadata_by_kind(NFTMetadataKind::NFT721, Maybe::Some(0u64), Maybe::None);
    assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);
}
#[test]
fn should_set_issuer() {
    let (mut contract, env) = default_token();

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_token_issuer = contract.get_token_issuer(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_issuer, token_owner);
}

#[test]
fn should_set_issuer_with_different_owner() {
    let (mut contract, env) = default_token();

    let token_issuer = env.get_account(0);
    let token_owner = env.get_account(1);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_token_issuer = contract.get_token_issuer(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_issuer, token_issuer);
}

#[test]
fn should_track_token_balance_by_owner() {
    let (mut contract, env) = default_token();

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_minter_balance = contract.balance_of(token_owner);
    let expected_minter_balance = 1u64;
    assert_eq!(actual_minter_balance, expected_minter_balance);
}

#[test]
fn should_allow_public_minting_with_flag_set_to_true() {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Public)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let account_1 = env.get_account(1);
    let minting_mode = contract.get_minting_mode();

    assert_eq!(
        minting_mode,
        MintingMode::Public,
        "public minting should be set to true"
    );

    let metadata = TEST_PRETTY_721_META_DATA.to_string();
    env.set_caller(account_1);
    contract.mint(account_1, metadata, Maybe::None);

    let token_id = 0u64;
    let minter_account_hash = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(account_1, minter_account_hash);
}

#[test]
fn should_disallow_public_minting_with_flag_set_to_false() {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Installer)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let account_1 = env.get_account(1);
    let metadata = TEST_PRETTY_721_META_DATA.to_string();

    let minting_mode = contract.get_minting_mode();
    assert_eq!(
        minting_mode,
        MintingMode::Installer,
        "public minting should be set to false"
    );

    env.set_caller(account_1);
    assert_eq!(
        contract.try_mint(account_1, metadata, Maybe::None),
        Err(CEP78Error::InvalidMinter.into()),
        "should not allow minting when minting is disabled"
    );
}

#[test]
fn should_allow_minting_for_different_public_key_with_minting_mode_set_to_public() {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Public)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let account_1 = env.get_account(1);
    let account_2 = env.get_account(2);

    let minting_mode = contract.get_minting_mode();
    assert_eq!(
        minting_mode,
        MintingMode::Public,
        "public minting should be set to true"
    );

    let metadata = TEST_PRETTY_721_META_DATA.to_string();
    assert!(contract
        .try_mint(account_1, metadata.clone(), Maybe::None)
        .is_ok());
    assert!(contract.try_mint(account_2, metadata, Maybe::None).is_ok());
}

#[test]
fn should_set_approval_for_all() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let owner = env.get_account(0);
    contract.mint(owner, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None);

    let operator = env.get_account(1);
    contract.set_approval_for_all(true, operator);

    let is_operator = contract.is_approved_for_all(owner, operator);
    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner, operator);
    assert!(env.emitted_event(contract.address(), &expected_event));

    // Test if two minted tokens are transferable by operator
    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    // Transfer first minted token by operator
    let result = contract.try_transfer(Maybe::Some(token_id), Maybe::None, owner, token_receiver);
    assert!(result.is_ok());

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);

    // Second mint by owner
    contract.mint(owner, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None);

    let token_id = 1u64;
    // Transfer second minted token by operator
    let result = contract.try_transfer(Maybe::Some(token_id), Maybe::None, owner, token_receiver);
    assert!(result.is_ok());

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
fn should_revoke_approval_for_all() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let owner = env.get_account(0);
    contract.mint(owner, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None);

    let operator = env.get_account(1);
    assert!(contract.try_set_approval_for_all(true, operator).is_ok());

    let is_operator = contract.is_approved_for_all(owner, operator);
    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner, operator);
    assert!(env.emitted_event(contract.address(), &expected_event),);

    assert!(contract.try_set_approval_for_all(false, operator).is_ok());

    let is_operator = contract.is_approved_for_all(owner, operator);
    assert!(!is_operator, "expected operator not to be approved for all");

    // Expect RevokedForAll event.
    let expected_event = RevokedForAll::new(owner, operator);
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn should_not_mint_with_invalid_nft721_metadata() {
    let (mut contract, env) = default_token();
    assert_eq!(
        contract.try_mint(
            env.get_account(0),
            MALFORMED_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::FailedToParse721Metadata.into()),
        "should not mint with invalid metadata"
    );
}

#[test]
fn should_mint_with_compactified_metadata() {
    let (mut contract, env) = default_token();
    contract.register_owner(Maybe::Some(env.get_account(0)));
    contract.mint(
        env.get_account(0),
        TEST_COMPACT_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_metadata =
        contract.get_metadata_by_kind(NFTMetadataKind::NFT721, Maybe::Some(token_id), Maybe::None);
    assert_eq!(TEST_PRETTY_721_META_DATA, actual_metadata);
}

#[test]
fn should_mint_with_valid_cep78_metadata() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    contract.mint(
        env.get_account(0),
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_metadata =
        contract.get_metadata_by_kind(NFTMetadataKind::CEP78, Maybe::Some(token_id), Maybe::None);
    assert_eq!(TEST_PRETTY_CEP78_METADATA, actual_metadata)
}

#[test]
fn should_mint_with_custom_metadata_validation() {
    let env = odra_test::env();
    let custom_json_schema =
        serde_json::to_string(&*TEST_CUSTOM_METADATA_SCHEMA).expect("must convert to json schema");
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::CustomValidated)
        .json_schema(custom_json_schema)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let metadata =
        serde_json::to_string(&*TEST_CUSTOM_METADATA).expect("must convert to json metadata");
    contract.mint(env.get_account(0), metadata, Maybe::None);

    let token_id = 0u64;
    let actual_metadata = contract.get_metadata_by_kind(
        NFTMetadataKind::CustomValidated,
        Maybe::Some(token_id),
        Maybe::None
    );
    let pretty_custom_metadata = serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA)
        .expect("must convert to json metadata");
    assert_eq!(pretty_custom_metadata, actual_metadata)
}

#[test]
fn should_mint_with_raw_metadata() {
    let env = odra_test::env();
    let args = default_args_builder()
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    contract.mint(env.get_account(0), "raw_string".to_string(), Maybe::None);

    let token_id = 0u64;
    let actual_metadata =
        contract.get_metadata_by_kind(NFTMetadataKind::Raw, Maybe::Some(token_id), Maybe::None);
    assert_eq!("raw_string".to_string(), actual_metadata)
}

#[test]
fn should_mint_with_hash_identifier_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );
    let token_id_hash =
        base16::encode_lower(&utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA));

    let token_page = contract.get_page_by_token_hash(token_id_hash);
    assert!(token_page[0]);
}

#[test]
fn should_fail_to_mint_when_immediate_caller_is_account_in_contract_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Unlocked)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    assert_eq!(
        contract.try_mint(
            env.get_account(0),
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::InvalidHolderMode.into())
    );
}

#[test]
fn should_approve_in_hash_identifier_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .metadata_mutability(MetadataMutability::Immutable)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    contract.mint(
        env.get_account(0),
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );
    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA);
    let token_hash = base16::encode_lower(&blake2b_hash);
    let spender = env.get_account(1);
    contract.approve(spender, Maybe::None, Maybe::Some(token_hash.clone()));

    let approved_account = contract.get_approved(Maybe::None, Maybe::Some(token_hash.clone()));
    assert_eq!(approved_account, Some(spender))
}

#[test]
fn should_mint_without_returning_receipts_and_flat_gas_cost() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    contract.mint(env.get_account(0), "".to_string(), Maybe::None);
    contract.mint(env.get_account(1), "".to_string(), Maybe::None);
    contract.mint(env.get_account(2), "".to_string(), Maybe::None);

    let costs = utils::get_gas_cost_of(&env, "mint");

    // In this case there is no first time allocation of a page.
    // Therefore the second and first mints must have equivalent gas costs.
    if let (Some(c1), Some(c2)) = (costs.get(1), costs.get(2)) {
        assert_eq!(c1, c2);
    }
}

// A test to ensure that the page table allocation is preserved
// even if the "register_owner" is called twice.
#[test]
fn should_maintain_page_table_despite_invoking_register_owner() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(token_owner, "".to_string(), Maybe::None);

    let actual_page_table = contract.get_page_table();
    assert_eq!(actual_page_table.len(), 1);

    // The mint WASM will register the owner, now we re-invoke the same entry point
    // and ensure that the page table doesn't mutate.
    contract.register_owner(Maybe::Some(token_owner));

    let table_post_register = contract.get_page_table();
    assert_eq!(actual_page_table, table_post_register)
}

#[test]
fn should_prevent_mint_to_unregistered_owner() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    assert_eq!(
        contract.try_mint(env.get_account(0), "".to_string(), Maybe::None),
        Err(CEP78Error::UnregisteredOwnerInMint.into())
    );
}

#[test]
fn should_mint_with_two_required_metadata_kind() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .additional_required_metadata(vec![NFTMetadataKind::Raw])
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let meta_78 =
        contract.get_metadata_by_kind(NFTMetadataKind::CEP78, Maybe::Some(token_id), Maybe::None);
    let meta_raw =
        contract.get_metadata_by_kind(NFTMetadataKind::Raw, Maybe::Some(token_id), Maybe::None);

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
    assert_eq!(meta_raw, TEST_PRETTY_CEP78_METADATA);
}

#[test]
fn should_mint_with_one_required_one_optional_metadata_kind_without_optional() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .optional_metadata(vec![NFTMetadataKind::Raw])
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let meta_78 =
        contract.get_metadata_by_kind(NFTMetadataKind::CEP78, Maybe::Some(token_id), Maybe::None);
    let meta_raw =
        contract.get_metadata_by_kind(NFTMetadataKind::Raw, Maybe::Some(token_id), Maybe::None);

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
    assert_eq!(meta_raw, TEST_PRETTY_CEP78_METADATA);

    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );

    let token_id = 1u64;
    let meta_78 =
        contract.get_metadata_by_kind(NFTMetadataKind::CEP78, Maybe::Some(token_id), Maybe::None);

    assert_eq!(meta_78, TEST_PRETTY_CEP78_METADATA);
}

#[test]
fn should_not_mint_with_missing_required_metadata() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .additional_required_metadata(vec![NFTMetadataKind::NFT721])
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);

    contract.register_owner(Maybe::Some(token_owner));
    assert_eq!(
        contract.try_mint(
            token_owner,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::FailedToParseCep78Metadata.into())
    );
}

#[test]
fn should_mint_with_transfer_only_reporting() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.mint(
        token_owner,
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );

    let actual_balance_after_mint = contract.balance_of(token_owner);
    let expected_balance_after_mint = 1u64;
    assert_eq!(actual_balance_after_mint, expected_balance_after_mint);
}

#[test]
fn should_approve_all_in_hash_identifier_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    let operator = env.get_account(1);
    contract.mint(
        token_owner,
        TEST_PRETTY_CEP78_METADATA.to_string(),
        Maybe::None
    );
    contract.mint(
        token_owner,
        TEST_PRETTY_UPDATED_CEP78_METADATA.to_string(),
        Maybe::None
    );

    contract.set_approval_for_all(true, operator);

    let is_operator = contract.is_approved_for_all(token_owner, operator);
    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(token_owner, operator);
    assert!(
        env.emitted_event(contract.address(), &expected_event),
        "Expected ApprovalForAll event."
    );
}

#[test]
fn should_approve_all_with_flat_gas_cost() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    let operator = env.get_account(1);
    let operator1 = env.get_account(2);
    let operator2 = env.get_account(3);

    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );
    contract.set_approval_for_all(true, operator);

    contract.set_approval_for_all(true, operator1);
    let is_operator = contract.is_approved_for_all(token_owner, operator1);
    assert!(is_operator, "expected operator to be approved for all");

    contract.set_approval_for_all(true, operator2);
    let is_also_operator = contract.is_approved_for_all(token_owner, operator2);
    assert!(
        is_also_operator,
        "expected other operator to be approved for all"
    );
    let costs = utils::get_gas_cost_of(&env, "set_approval_for_all");

    // Operator approval should have flat gas costs.
    // First call creates necessary named keys.
    // Therefore the second and third set_approve_for_all must have equivalent gas costs.
    assert_eq!(costs.get(1), costs.get(2));
}
