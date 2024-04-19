use odra::{
    args::Maybe,
    host::{Deployer, HostRef}
};

use crate::cep78::{
    modalities::{EventsMode, OwnerReverseLookupMode, OwnershipMode},
    tests::{default_args_builder, utils::TEST_PRETTY_721_META_DATA},
    token::CEP78HostRef
};

// cep47 event style
#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_record_cep47_dictionary_style_mint_event() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_record_cep47_dictionary_style_transfer_token_event_in_hash_identifier_mode() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_record_cep47_dictionary_style_metadata_update_event_for_nft721_using_token_id() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_cep47_dictionary_style_burn_event() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_cep47_dictionary_style_approve_event_in_hash_identifier_mode() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_cep47_dictionary_style_approvall_for_all_event() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_cep47_dictionary_style_revoked_for_all_event() {}

#[test]
#[ignore = "Odra does not support cep47 events"]
fn should_record_migration_event_in_cep47() {}

#[test]
#[ignore = "Named keys existence is not verifiable in Odra"]
fn should_not_have_events_dicts_in_no_events_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .events_mode(EventsMode::NoEvents)
        .build();
    let _contract = CEP78HostRef::deploy(&env, args);

    // Check dict from EventsMode::CES
    // let events = named_keys.get(EVENTS_DICT);
    // assert_eq!(events, None);
}

#[test]
fn should_not_record_events_in_no_events_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .events_mode(EventsMode::NoEvents)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    contract.mint(
        token_owner,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    // This will error if token is not registered as burnt
    let actual_balance = contract.balance_of(token_owner);
    let expected_balance = 1u64;
    assert_eq!(actual_balance, expected_balance);

    assert!(env.events_count(contract.address()) == 0);
}
