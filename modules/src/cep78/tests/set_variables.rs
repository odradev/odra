use odra::{
    args::Maybe,
    host::{Deployer, HostRef}
};

use crate::cep78::{
    error::CEP78Error, events::VariablesSet, modalities::EventsMode, tests::default_args_builder,
    token::TestCep78HostRef
};

#[test]
fn only_installer_should_be_able_to_toggle_allow_minting() {
    let env = odra_test::env();
    let (installer, other_user) = (env.get_account(0), env.get_account(1));
    let args = default_args_builder()
        .allow_minting(false)
        .events_mode(EventsMode::CES)
        .build();
    let mut contract = TestCep78HostRef::deploy(&env, args);

    // Account other than installer account should not be able to change allow_minting
    env.set_caller(other_user);
    assert_eq!(
        contract.try_set_variables(Maybe::Some(true), Maybe::None, Maybe::None),
        Err(CEP78Error::InvalidAccount.into())
    );
    assert!(!contract.is_minting_allowed());

    // Installer account should be able to change allow_minting
    env.set_caller(installer);
    assert_eq!(
        contract.try_set_variables(Maybe::Some(true), Maybe::None, Maybe::None),
        Ok(())
    );
    assert!(contract.is_minting_allowed());

    // Expect VariablesSet event.
    assert!(env.emitted_event(contract.address(), &VariablesSet {}));
}

#[test]
#[ignore = "Acl package mode is true by default and immutable"]
fn installer_should_be_able_to_toggle_acl_package_mode() {}

#[test]
#[ignore = "Package operator mode is true by default and immutable"]
fn installer_should_be_able_to_toggle_package_operator_mode() {}

#[test]
fn installer_should_be_able_to_toggle_operator_burn_mode() {
    let env = odra_test::env();
    let args = default_args_builder().events_mode(EventsMode::CES).build();
    let mut contract = TestCep78HostRef::deploy(&env, args);

    // Installer account should be able to change allow_minting
    assert_eq!(
        contract.try_set_variables(Maybe::None, Maybe::None, Maybe::Some(true)),
        Ok(())
    );
    assert!(contract.is_operator_burn_mode());

    // Expect VariablesSet event.
    assert!(env.emitted(contract.address(), "VariablesSet"));
}
