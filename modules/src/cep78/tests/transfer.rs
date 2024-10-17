use odra::{
    args::Maybe,
    host::{Deployer, HostEnv, HostRef, NoArgs},
    prelude::*
};

use crate::cep78::{
    error::CEP78Error,
    events::{Approval, ApprovalRevoked, Transfer},
    modalities::{
        EventsMode, MetadataMutability, MintingMode, NFTHolderMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, TokenIdentifier,
        TransferFilterContractResult, WhitelistMode
    },
    tests::{default_args_builder, utils::TEST_PRETTY_721_META_DATA},
    token::TestCep78,
    utils::{MockCep78Operator, MockCep78TransferFilter}
};

use super::utils;

#[test]
fn should_disallow_transfer_with_minter_or_assigned_ownership_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Assigned)
        .minting_mode(MintingMode::Installer)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_owner_balance = contract.balance_of(token_owner);
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        ),
        Err(CEP78Error::InvalidOwnershipMode.into())
    );
}

#[test]
fn should_transfer_token_from_sender_to_receiver() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_owner_balance = contract.balance_of(token_owner);
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    contract.transfer(
        Maybe::Some(token_id),
        Maybe::None,
        token_owner,
        token_receiver
    );

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);

    env.set_caller(token_receiver);
    let token_receiver_page = contract.get_page_by_token_id(token_id);
    assert!(token_receiver_page[0]);

    let actual_sender_balance = contract.balance_of(token_owner);
    let expected_sender_balance = 0u64;
    assert_eq!(actual_sender_balance, expected_sender_balance);

    let actual_receiver_balance = contract.balance_of(token_receiver);
    let expected_receiver_balance = 1u64;
    assert_eq!(actual_receiver_balance, expected_receiver_balance);

    // Expect Transfer event.
    let expected_event = Transfer::new(
        token_owner,
        None,
        token_receiver,
        TokenIdentifier::Index(token_id).to_string()
    );
    assert!(env.emitted_event(contract.address(), &expected_event));
}

fn approve_token_for_transfer_should_add_entry_to_approved_dictionary(
    env: HostEnv,
    operator: Option<Address>
) {
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let spender = env.get_account(1);
    let token_id = 0u64;

    if let Some(operator) = operator {
        assert!(contract.try_set_approval_for_all(true, operator).is_ok());
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => token_owner
    };
    env.set_caller(approving_account);
    assert!(contract
        .try_approve(spender, Maybe::Some(token_id), Maybe::None)
        .is_ok());

    let actual_approved_key = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_approved_key, Some(spender));

    // Expect Approval event.
    let expected_event = Approval::new(
        token_owner,
        spender,
        TokenIdentifier::Index(token_id).to_string()
    );
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn approve_token_for_transfer_from_an_account_should_add_entry_to_approved_dictionary() {
    let env = odra_test::env();
    approve_token_for_transfer_should_add_entry_to_approved_dictionary(env, None)
}

#[test]
fn approve_token_for_transfer_from_an_operator_should_add_entry_to_approved_dictionary() {
    let env = odra_test::env();
    let operator = env.get_account(10);
    approve_token_for_transfer_should_add_entry_to_approved_dictionary(env, Some(operator))
}

fn revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(
    env: HostEnv,
    operator: Option<Address>
) {
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let spender = env.get_account(1);
    let token_id = 0u64;

    if let Some(operator) = operator {
        assert!(contract.try_set_approval_for_all(true, operator).is_ok());
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => token_owner
    };
    env.set_caller(approving_account);
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    let actual_approved_key = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_approved_key, Some(spender));

    env.set_caller(token_owner);
    contract.revoke(Maybe::Some(token_id), Maybe::None);

    let actual_approved_key = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_approved_key, None);

    // Expect ApprovalRevoked event.
    let expected_event =
        ApprovalRevoked::new(token_owner, TokenIdentifier::Index(token_id).to_string());
    assert!(env.emitted_event(contract.address(), &expected_event));
}

#[test]
fn revoke_token_for_transfer_from_account_should_remove_entry_to_approved_dictionary() {
    revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(odra_test::env(), None)
}

#[test]
fn revoke_token_for_transfer_from_operator_should_remove_entry_to_approved_dictionary() {
    let env = odra_test::env();
    let operator = env.get_account(10);
    revoke_token_for_transfer_should_remove_entry_to_approved_dictionary(env, Some(operator))
}

#[test]
fn should_disallow_approving_when_ownership_mode_is_minter_or_assigned() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Assigned)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let spender = env.get_account(1);
    let token_id = 0u64;

    assert_eq!(
        contract.try_approve(spender, Maybe::Some(token_id), Maybe::None),
        Err(CEP78Error::InvalidOwnershipMode.into())
    );
}

fn should_be_able_to_transfer_token(env: HostEnv, operator: Option<Address>) {
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    // Create a "to approve" spender account account and transfer funds
    let spender = env.get_account(1);
    let token_id = 0u64;

    if let Some(operator) = operator {
        assert!(contract.try_set_approval_for_all(true, operator).is_ok());
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => token_owner
    };
    env.set_caller(approving_account);
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    let actual_approved_key = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_approved_key, Some(spender));

    // Create to_account and transfer minted token using spender
    let to_account = env.get_account(2);
    contract.register_owner(Maybe::Some(to_account));
    contract.transfer(Maybe::Some(token_id), Maybe::None, token_owner, to_account);

    let actual_approved_account_hash = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(
        actual_approved_account_hash, None,
        "approved account should be set to none after a transfer"
    );
}

#[test]
fn should_be_able_to_transfer_token_using_approved_account() {
    should_be_able_to_transfer_token(odra_test::env(), None)
}

#[test]
fn should_be_able_to_transfer_token_using_operator() {
    let env = odra_test::env();
    let operator = env.get_account(11);
    should_be_able_to_transfer_token(env, Some(operator))
}

#[test]
fn should_disallow_same_approved_account_to_transfer_token_twice() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_owner_balance = contract.balance_of(token_owner);
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    // Create a "to approve" spender account and transfer funds
    let spender = env.get_account(1);
    let token_id = 0u64;

    // Approve spender
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    let actual_approved_account = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    let expected_approved_account = Some(spender);
    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );

    // Create to_account and transfer minted token using spender
    let to_account = env.get_account(2);
    contract.register_owner(Maybe::Some(to_account));

    env.set_caller(spender);
    contract.transfer(Maybe::Some(token_id), Maybe::None, token_owner, to_account);

    // Create to_other_account and transfer minted token using spender
    let to_other_account = env.get_account(3);
    contract.register_owner(Maybe::Some(to_other_account));

    env.set_caller(spender);
    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            to_account,
            to_other_account
        ),
        Err(CEP78Error::InvalidTokenOwner.into())
    );
}

fn should_disallow_to_transfer_token_using_revoked_hash(env: HostEnv, operator: Option<Address>) {
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_owner_balance = contract.balance_of(token_owner);
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    // Create a "to approve" spender account and transfer funds
    let spender = env.get_account(1);
    let token_id = 0u64;

    if let Some(operator) = operator {
        assert!(contract.try_set_approval_for_all(true, operator).is_ok());
    }

    let approving_account = match operator {
        Some(operator) => operator,
        None => token_owner
    };
    env.set_caller(approving_account);
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    let actual_approved_account = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    let expected_approved_account = Some(spender);
    assert_eq!(
        actual_approved_account, expected_approved_account,
        "approved account should have been set in dictionary when approved"
    );

    // Create to_account and transfer minted token using account
    let to_account = env.get_account(2);
    contract.register_owner(Maybe::Some(to_account));

    // Revoke approval
    contract.revoke(Maybe::Some(token_id), Maybe::None);

    env.set_caller(spender);
    assert_eq!(
        contract.try_transfer(Maybe::Some(token_id), Maybe::None, token_owner, to_account),
        Err(CEP78Error::InvalidTokenOwner.into())
    );

    let actual_approved_account_hash = contract.get_approved(Maybe::Some(token_id), Maybe::None);
    assert_eq!(
        actual_approved_account_hash, None,
        "approved account should be unset after revoke and a failed transfer"
    );
}

#[test]
fn should_disallow_to_transfer_token_using_revoked_account() {
    should_disallow_to_transfer_token_using_revoked_hash(odra_test::env(), None)
}

#[test]
fn should_disallow_to_transfer_token_using_revoked_operator() {
    let env = odra_test::env();
    let operator = env.get_account(11);
    should_disallow_to_transfer_token_using_revoked_hash(env, Some(operator))
}

#[test]
#[ignore = "Odra does not support deprecated arguments"]
fn should_be_able_to_approve_with_deprecated_operator_argument() {}

#[test]
fn should_transfer_between_contract_to_account() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .events_mode(EventsMode::CES)
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );
    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), true);

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(
        &actual_token_owner,
        minting_contract.address(),
        "token owner is not minting contract"
    );

    let receiver = env.get_account(0);
    contract.register_owner(Maybe::Some(receiver));
    minting_contract.transfer(token_id, receiver);

    let updated_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(updated_token_owner, receiver);
}

#[test]
fn should_prevent_transfer_when_caller_is_not_owner() {
    let env = odra_test::env();
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Accounts)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    let unauthorized_user = env.get_account(10);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(token_owner, actual_token_owner);

    env.set_caller(unauthorized_user);
    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            unauthorized_user
        ),
        Err(CEP78Error::InvalidTokenOwner.into())
    );
}

#[test]
fn should_transfer_token_in_hash_identifier_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .identifier_mode(NFTIdentifierMode::Hash)
        .ownership_mode(OwnershipMode::Transferable)
        .metadata_mutability(MetadataMutability::Immutable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

    let token_owner = env.get_account(0);
    let new_owner = env.get_account(1);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let blake2b_hash = utils::create_blake2b_hash(TEST_PRETTY_721_META_DATA);
    let token_hash = base16::encode_lower(&blake2b_hash);
    contract.register_owner(Maybe::Some(new_owner));
    assert!(contract
        .try_transfer(Maybe::None, Maybe::Some(token_hash), token_owner, new_owner)
        .is_ok());
}

#[test]
fn should_not_allow_non_approved_contract_to_transfer() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .holder_mode(NFTHolderMode::Mixed)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    assert_eq!(
        minting_contract.try_transfer_from(token_id, token_owner, token_receiver),
        Err(CEP78Error::InvalidTokenOwner.into())
    );

    contract.approve(
        *minting_contract.address(),
        Maybe::Some(token_id),
        Maybe::None
    );
    assert!(minting_contract
        .try_transfer_from(token_id, token_owner, token_receiver)
        .is_ok());
}

#[test]
fn transfer_should_correctly_track_page_table_entries() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    let new_owner = env.get_account(1);

    let number_of_tokens_pre_migration = 20usize;
    for _ in 0..number_of_tokens_pre_migration {
        contract.register_owner(Maybe::Some(token_owner));
        contract.mint(token_owner, "".to_string(), Maybe::None);
    }

    contract.register_owner(Maybe::Some(new_owner));
    contract.transfer(Maybe::Some(11u64), Maybe::None, token_owner, new_owner);

    env.set_caller(new_owner);
    let token_owner_page_table = contract.get_page_table();
    assert!(token_owner_page_table[0])
}

#[test]
fn should_prevent_transfer_to_unregistered_owner() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .metadata_mutability(MetadataMutability::Immutable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(token_owner, "".to_string(), Maybe::None);

    let token_id = 0u64;
    let token_receiver = env.get_account(1);
    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        ),
        Err(CEP78Error::UnregisteredOwnerInTransfer.into())
    );
}

#[test]
fn should_transfer_token_from_sender_to_receiver_with_transfer_only_reporting() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::TransfersOnly)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    env.set_caller(token_owner);
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let actual_owner_balance = contract.balance_of(token_owner);
    let expected_owner_balance = 1u64;
    assert_eq!(actual_owner_balance, expected_owner_balance);

    let token_receiver = env.get_account(1);
    env.set_caller(token_owner);
    contract.register_owner(Maybe::Some(token_receiver));
    contract.transfer(Maybe::Some(0u64), Maybe::None, token_owner, token_receiver);

    let actual_token_owner = contract.owner_of(Maybe::Some(0u64), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);

    env.set_caller(token_receiver);
    let token_receiver_page = contract.get_page_by_token_id(0u64);
    assert!(token_receiver_page[0]);

    let actual_sender_balance = contract.balance_of(token_owner);
    let expected_sender_balance = 0u64;
    assert_eq!(actual_sender_balance, expected_sender_balance);

    let actual_receiver_balance = contract.balance_of(token_receiver);
    let expected_receiver_balance = 1u64;
    assert_eq!(actual_receiver_balance, expected_receiver_balance);
}

#[test]
fn disallow_owner_to_approve_itself() {
    let env = odra_test::env();
    let args = default_args_builder()
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

    assert_eq!(
        contract.try_approve(token_owner, Maybe::Some(0u64), Maybe::None),
        Err(CEP78Error::InvalidAccount.into())
    );
}

#[test]
fn disallow_operator_to_approve_itself() {
    let env = odra_test::env();
    let args = default_args_builder()
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

    let token_id = 0u64;
    let operator = env.get_account(1);

    assert!(contract.try_set_approval_for_all(true, operator).is_ok());
    env.set_caller(operator);
    assert_eq!(
        contract.try_approve(operator, Maybe::Some(token_id), Maybe::None),
        Err(CEP78Error::InvalidAccount.into())
    );
}

#[test]
fn disallow_owner_to_approve_for_all_itself() {
    let env = odra_test::env();
    let args = default_args_builder()
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

    assert_eq!(
        contract.try_set_approval_for_all(true, token_owner),
        Err(CEP78Error::InvalidAccount.into())
    );
}

#[test]
fn check_transfers_with_transfer_filter_contract_modes() {
    let env = odra_test::env();

    let mut transfer_filter_contract = MockCep78TransferFilter::deploy(&env, NoArgs);
    transfer_filter_contract.set_return_value(TransferFilterContractResult::DenyTransfer as u8);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .transfer_filter_contract_contract_key(*transfer_filter_contract.address())
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_owner = env.get_account(0);

    for _i in 0..2 {
        contract.mint(
            token_owner,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        );
    }

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        ),
        Err(CEP78Error::TransferFilterContractDenied.into())
    );

    transfer_filter_contract.set_return_value(TransferFilterContractResult::ProceedTransfer as u8);
    assert!(contract
        .try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        )
        .is_ok());

    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_receiver,
            token_owner
        ),
        Err(CEP78Error::InvalidTokenOwner.into())
    );

    let token_id = 1u64;
    assert!(contract
        .try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        )
        .is_ok());
}

#[test]
fn should_disallow_transfer_from_contract_with_package_operator_mode_without_operator() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Mixed)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    assert_eq!(
        minting_contract.try_transfer_from(token_id, token_owner, token_receiver),
        Err(CEP78Error::InvalidTokenOwner.into())
    );
}

#[test]
#[ignore = "Odra does not support package operator mode - is always on"]
fn should_disallow_transfer_from_contract_without_package_operator_mode_with_package_as_operator() {
}

#[test]
fn should_allow_transfer_from_contract_with_package_operator_mode_with_operator() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Mixed)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));

    let token_id = 0u64;
    contract.set_approval_for_all(true, *minting_contract.address());
    assert!(minting_contract
        .try_transfer_from(token_id, token_owner, token_receiver)
        .is_ok());

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
#[ignore = "Odra does not support package operator mode - is always on"]
fn should_disallow_package_operator_to_approve_without_package_operator_mode() {}

#[test]
fn should_allow_package_operator_to_approve_with_package_operator_mode() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Mixed)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));
    contract.set_approval_for_all(true, *minting_contract.address());

    let token_id = 0u64;
    let spender = env.get_account(2);
    minting_contract.approve(spender, token_id);

    env.set_caller(spender);
    contract.transfer(
        Maybe::Some(token_id),
        Maybe::None,
        token_owner,
        token_receiver
    );

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
fn should_allow_account_to_approve_spender_with_package_operator() {
    let env = odra_test::env();
    let minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Mixed)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));
    contract.set_approval_for_all(true, *minting_contract.address());

    let token_id = 0u64;
    let spender = env.get_account(2);
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    env.set_caller(spender);
    contract.transfer(
        Maybe::Some(token_id),
        Maybe::None,
        token_owner,
        token_receiver
    );

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
fn should_allow_package_operator_to_revoke_with_package_operator_mode() {
    let env = odra_test::env();
    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let args = default_args_builder()
        .ownership_mode(OwnershipMode::Transferable)
        .holder_mode(NFTHolderMode::Mixed)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_receiver = env.get_account(1);
    contract.register_owner(Maybe::Some(token_receiver));
    contract.set_approval_for_all(true, *minting_contract.address());

    let token_id = 0u64;
    let spender = env.get_account(2);
    contract.approve(spender, Maybe::Some(token_id), Maybe::None);

    minting_contract.revoke(token_id);
    env.set_caller(spender);

    assert_eq!(
        contract.try_transfer(
            Maybe::Some(token_id),
            Maybe::None,
            token_owner,
            token_receiver
        ),
        Err(CEP78Error::InvalidTokenOwner.into())
    );

    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, token_owner);
}
