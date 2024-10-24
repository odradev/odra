use odra::{
    args::Maybe,
    host::{Deployer, HostRef, NoArgs},
    prelude::*
};

use crate::cep78::{
    error::CEP78Error,
    events::Burn,
    modalities::{
        BurnMode, EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        OwnerReverseLookupMode, OwnershipMode, TokenIdentifier
    },
    tests::{default_args_builder, utils},
    token::{TestCep78, TestCep78HostRef},
    utils::MockCep78Operator
};

use super::utils::TEST_PRETTY_721_META_DATA;

fn should_burn_minted_token(reporting: OwnerReverseLookupMode) {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(reporting)
        .ownership_mode(OwnershipMode::Transferable)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    let token_id = 0u64;
    let reverse_lookup_enabled = reporting == OwnerReverseLookupMode::Complete;
    mint(&mut contract, reverse_lookup_enabled, token_owner);

    let actual_balance_before_burn = contract.balance_of(token_owner);
    let expected_balance_before_burn = 1u64;
    assert_eq!(actual_balance_before_burn, expected_balance_before_burn);

    let burn_result = contract.try_burn(Maybe::Some(token_id), Maybe::None);
    assert!(burn_result.is_ok());

    // This will error if token is not registered as burnt.
    assert!(contract.token_burned(Maybe::Some(token_id), Maybe::None));

    // This will error if token is not registered as burnt
    let actual_balance = contract.balance_of(token_owner);
    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    // Expect Burn event.
    let expected_event = Burn::new(
        token_owner,
        TokenIdentifier::Index(token_id).to_string(),
        token_owner
    );
    assert!(
        env.emitted_event(contract.address(), &expected_event),
        "Expected Burn event."
    );
}

fn mint(contract: &mut TestCep78HostRef, reverse_lookup_enabled: bool, token_owner: Address) {
    if reverse_lookup_enabled {
        contract.register_owner(Maybe::Some(token_owner));
        contract.mint(
            token_owner,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        );
        let token_page = contract.get_page_by_token_id(0u64);
        assert!(token_page[0]);
    } else {
        contract.mint(
            token_owner,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        );
    }
}

#[test]
fn should_burn_minted_token_with_complete_reporting() {
    should_burn_minted_token(OwnerReverseLookupMode::Complete);
}

#[test]
fn should_burn_minted_token_with_transfer_only_reporting() {
    should_burn_minted_token(OwnerReverseLookupMode::TransfersOnly);
}

#[test]
fn should_not_burn_previously_burnt_token() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .ownership_mode(OwnershipMode::Transferable)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    mint(&mut contract, true, token_owner);

    let burn_result = contract.try_burn(Maybe::Some(0u64), Maybe::None);
    assert!(burn_result.is_ok());

    let re_burn_result = contract.try_burn(Maybe::Some(0u64), Maybe::None);
    assert_eq!(
        re_burn_result,
        Err(CEP78Error::PreviouslyBurntToken.into()),
        "should disallow burning of previously burnt token"
    );
}

#[test]
fn should_return_expected_error_when_burning_non_existing_token() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_id = 0u64;
    assert_eq!(
        contract.try_burn(Maybe::Some(token_id), Maybe::None),
        Err(CEP78Error::MissingOwnerTokenIdentifierKey.into()),
        "should return InvalidTokenID error when trying to burn a non_existing token",
    );
}

#[test]
fn should_return_expected_error_burning_of_others_users_token() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    let account_1 = env.get_account(1);

    mint(&mut contract, true, token_owner);
    let token_id = 0u64;
    env.set_caller(account_1);
    assert_eq!(
        contract.try_burn(Maybe::Some(token_id), Maybe::None),
        Err(CEP78Error::InvalidTokenOwner.into()),
        "should return InvalidTokenID error when trying to burn a non_existing token",
    );
}

#[test]
fn should_allow_contract_to_burn_token() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .ownership_mode(OwnershipMode::Transferable)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    minting_contract.mint_for(token_owner, TEST_PRETTY_721_META_DATA.to_string());

    let current_token_balance = contract.balance_of(token_owner);
    assert_eq!(1u64, current_token_balance);

    contract.burn(Maybe::Some(0u64), Maybe::None);

    let updated_token_balance = contract.balance_of(token_owner);
    assert_eq!(updated_token_balance, 0u64)
}

#[test]
fn should_not_burn_in_non_burn_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .burn_mode(BurnMode::NonBurnable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    mint(&mut contract, true, token_owner);

    assert_eq!(
        contract.try_burn(Maybe::Some(0u64), Maybe::None),
        Err(CEP78Error::InvalidBurnMode.into())
    );
}

#[test]
fn should_let_account_operator_burn_tokens_with_operator_burn_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .operator_burn_mode(true)
        .events_mode(EventsMode::CES)
        .build();

    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    mint(&mut contract, true, token_owner);

    let token_id = 0u64;
    let operator = env.get_account(1);

    env.set_caller(operator);
    assert_eq!(
        contract.try_burn(Maybe::Some(token_id), Maybe::None),
        Err(CEP78Error::InvalidTokenOwner.into()),
        "InvalidTokenOwner should not allow burn by non operator"
    );

    env.set_caller(token_owner);
    contract.set_approval_for_all(true, operator);

    env.set_caller(operator);
    assert!(contract
        .try_burn(Maybe::Some(token_id), Maybe::None)
        .is_ok());
    assert!(contract.token_burned(Maybe::Some(token_id), Maybe::None));

    let actual_balance = contract.balance_of(token_owner);
    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let expected_event = Burn::new(
        token_owner,
        TokenIdentifier::Index(token_id).to_string(),
        operator
    );
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn should_let_contract_operator_burn_tokens_with_operator_burn_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .operator_burn_mode(true)
        .events_mode(EventsMode::CES)
        .build();

    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    mint(&mut contract, true, token_owner);

    let token_id = 0u64;
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    minting_contract.set_address(contract.address());
    let operator = *minting_contract.address();
    let account_1 = env.get_account(1);

    env.set_caller(account_1);
    assert_eq!(
        minting_contract.try_burn(token_id),
        Err(CEP78Error::InvalidTokenOwner.into()),
        "InvalidTokenOwner should not allow burn by non operator"
    );

    env.set_caller(token_owner);
    contract.set_approval_for_all(true, operator);
    env.set_caller(account_1);
    assert!(minting_contract.try_burn(token_id).is_ok());

    assert!(contract.token_burned(Maybe::Some(token_id), Maybe::None));

    let actual_balance = contract.balance_of(token_owner);
    let expected_balance = 0u64;
    assert_eq!(actual_balance, expected_balance);

    let expected_event = Burn::new(
        token_owner,
        TokenIdentifier::Index(token_id).to_string(),
        operator
    );
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn should_burn_token_in_hash_identifier_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .metadata_mutability(MetadataMutability::Immutable)
        .build();

    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    mint(&mut contract, true, token_owner);

    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA);
    let token_hash = base16::encode_lower(&blake2b_hash);

    assert!(contract
        .try_burn(Maybe::None, Maybe::Some(token_hash))
        .is_ok());
}

#[test]
fn should_allow_to_remint_with_reverse_lookup() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .ownership_mode(OwnershipMode::Transferable)
        .events_mode(EventsMode::CES)
        .identifier_mode(NFTIdentifierMode::Hash)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_hash = "token_hash".to_string();
    let token_owner = env.get_account(0);

    // Mint the token.
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::Some(token_hash.clone())
    );
    let token_page = contract.get_page_by_token_id(0);
    assert!(token_page[0]);
    assert_eq!(contract.balance_of(token_owner), 1);
    assert_eq!(contract.get_number_of_minted_tokens(), 1);

    // Burn the token.
    contract.burn(Maybe::None, Maybe::Some(token_hash.clone()));
    assert_eq!(contract.balance_of(token_owner), 0);

    // Mint the token again.
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::Some(token_hash.clone())
    );

    // Check if the token is minted.
    let token_page = contract.get_page_by_token_id(0);
    assert!(token_page[0]);
    assert_eq!(contract.balance_of(token_owner), 1);
    assert_eq!(contract.get_number_of_minted_tokens(), 1);
}
