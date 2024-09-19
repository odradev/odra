use odra::{
    args::Maybe,
    host::{Deployer, HostRef}
};

use crate::cep78::{
    modalities::{EventsMode, OwnerReverseLookupMode, OwnershipMode},
    tests::{default_args_builder, utils::TEST_PRETTY_721_META_DATA},
    token::TestCep78
};

#[test]
#[ignore = "Named keys existence is not verifiable in Odra"]
fn should_not_have_events_dicts_in_no_events_mode() {}

#[test]
fn should_not_record_events_in_no_events_mode() {
    let env = odra_test::env();
    let args = default_args_builder()
        .events_mode(EventsMode::NoEvents)
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

    // This will error if token is not registered as burnt
    let actual_balance = contract.balance_of(token_owner);
    let expected_balance = 1u64;
    assert_eq!(actual_balance, expected_balance);

    assert_eq!(0, env.events_count(contract.address()));
}
