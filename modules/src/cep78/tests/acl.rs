use odra::{
    args::Maybe,
    host::{Deployer, HostEnv, HostRef, NoArgs},
    prelude::*,
};

use crate::cep78::{
    error::CEP78Error,
    modalities::{MintingMode, NFTHolderMode, OwnershipMode, WhitelistMode},
    tests::utils::{DummyContractHostRef, TestContractHostRef, TEST_PRETTY_721_META_DATA},
    token::CEP78HostRef
};

use super::default_args_builder;

#[test]
fn should_install_with_acl_whitelist() {
    let env = odra_test::env();

    let test_contract_address = TestContractHostRef::deploy(&env, NoArgs);

    let contract_whitelist = vec![test_contract_address.address().clone()];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);

    assert_eq!(WhitelistMode::Locked, contract.get_whitelist_mode());
    let is_whitelisted_contract = contract.is_whitelisted(test_contract_address.address());
    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");
}

#[test]
#[ignore = "No need to implement a contract whitelist"]
fn should_install_with_deprecated_contract_whitelist() {}

#[test]
#[ignore = "Can't assert init errors in Odra"]
fn should_not_install_with_minting_mode_not_acl_if_acl_whitelist_provided() {
    let env = odra_test::env();

    let contract_whitelist = vec![env.get_account(0)];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Transferable)
        .minting_mode(MintingMode::Installer) // Not the right minting mode for acl
        .acl_white_list(contract_whitelist)
        .build();

    CEP78HostRef::deploy(&env, args);

    // builder.exec(install_request).expect_failure();

    // let actual_error = builder.get_error().expect("must have error");
    // support::assert_expected_error(
    //     actual_error,
    //     38u16,
    //     "should disallow installing without acl minting mode if non empty acl list",
    // );
}

fn should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
    env: &HostEnv,
    nft_holder_mode: NFTHolderMode
) {
    let args = default_args_builder()
        .holder_mode(nft_holder_mode)
        .whitelist_mode(WhitelistMode::Locked)
        .minting_mode(MintingMode::Public)
        .build();

    CEP78HostRef::deploy(env, args);
}

#[test]
fn should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode() {
    let env = odra_test::env();
    should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        &env,
        NFTHolderMode::Accounts,
    );
    should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
            &env,
    NFTHolderMode::Contracts,
    );
    should_allow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        &env,
        NFTHolderMode::Mixed
    );
}

// TODO: in Odra contract installation always succeeds, so this test is not applicable
#[test]
#[ignore = "in Odra contract installation always succeeds, so this test is not applicable"]
fn should_disallow_installation_with_contract_holder_mode_and_installer_mode() {
    let env = odra_test::env();
    let contract_whitelist = vec![env.get_account(1), env.get_account(2), env.get_account(3)];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Installer)
        .acl_white_list(contract_whitelist)
        .build();

    CEP78HostRef::deploy(&env, args);
    // builder.exec(install_request).expect_failure();
    // let error = builder.get_error().expect("should have an error");
    // assert_expected_error(error, 38, "Invalid MintingMode (not ACL) and NFTHolderMode");
}

#[test]
fn should_allow_whitelisted_account_to_mint() {
    let env = odra_test::env();

    let account_user_1 = env.get_account(1);
    let account_whitelist = vec![account_user_1];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Accounts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(account_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);

    assert!(
        contract.is_whitelisted(&account_user_1),
        "acl whitelist is incorrectly set"
    );

    env.set_caller(account_user_1);
    contract.mint(
        account_user_1,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);

    assert_eq!(actual_token_owner, account_user_1);
}

#[test]
fn should_disallow_unlisted_account_from_minting() {
    let env = odra_test::env();
    let account = env.get_account(0);
    let account_whitelist = vec![account];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Accounts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(account_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);

    assert!(
        contract.is_whitelisted(&account),
        "acl whitelist is incorrectly set"
    );
    let account_user_1 = env.get_account(1);

    env.set_caller(account_user_1);
    assert_eq!(
        contract.try_mint(
            account_user_1,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::InvalidMinter.into()),
        "Unlisted account hash should not be permitted to mint"
    );
}

#[test]
fn should_allow_whitelisted_contract_to_mint() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);

    let contract_whitelist = vec![minting_contract.address().clone()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);
    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(contract.address(), TEST_PRETTY_721_META_DATA.to_string());

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(&actual_token_owner, minting_contract.address())
}

#[test]
fn should_disallow_unlisted_contract_from_minting() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);

    let contract_whitelist = vec![env.get_account(1), env.get_account(2), env.get_account(3)];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);

    assert_eq!(
        minting_contract.try_mint(contract.address(), TEST_PRETTY_721_META_DATA.to_string(),),
        Err(CEP78Error::UnlistedContractHash.into()),
        "Unlisted account hash should not be permitted to mint"
    );
}

#[test]
fn should_allow_mixed_account_contract_to_mint() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);
    let account_user_1 = env.get_account(1);
    let mixed_whitelist = vec![minting_contract.address().clone(), account_user_1];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(contract.address(), TEST_PRETTY_721_META_DATA.to_string());

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(&actual_token_owner, minting_contract.address());

    assert!(
        contract.is_whitelisted(&account_user_1),
        "acl whitelist is incorrectly set"
    );
    env.set_caller(account_user_1);
    contract.mint(
        account_user_1,
        TEST_PRETTY_721_META_DATA.to_string(),
        Maybe::None
    );

    let token_id = 1u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(actual_token_owner, account_user_1)
}

#[test]
fn should_disallow_unlisted_contract_from_minting_with_mixed_account_contract() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);
    let account_user_1 = env.get_account(1);
    let mixed_whitelist = vec![
        DummyContractHostRef::deploy(&env, NoArgs).address().clone(),
        account_user_1,
    ];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);

    assert_eq!(
        minting_contract.try_mint(contract.address(), TEST_PRETTY_721_META_DATA.to_string(),),
        Err(CEP78Error::UnlistedContractHash.into()),
        "Unlisted contract should not be permitted to mint"
    );
}

#[test]
fn should_disallow_unlisted_account_from_minting_with_mixed_account_contract() {
    let env = odra_test::env();

    let minting_contract = TestContractHostRef::deploy(&env, NoArgs);
    let listed_account = env.get_account(0);
    let unlisted_account = env.get_account(1);
    let mixed_whitelist = vec![minting_contract.address().clone(), listed_account];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);
    env.set_caller(unlisted_account);
    assert_eq!(
        contract.try_mint(
            unlisted_account,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::InvalidMinter.into()),
        "Unlisted account should not be permitted to mint"
    );
}

#[test]
fn should_disallow_listed_account_from_minting_with_nftholder_contract() {
    let env = odra_test::env();

    let minting_contract = TestContractHostRef::deploy(&env, NoArgs);
    let listed_account = env.get_account(0);

    let mixed_whitelist = vec![minting_contract.address().clone(), listed_account];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);

    assert!(
        contract.is_whitelisted(&listed_account),
        "acl whitelist is incorrectly set"
    );

    let unlisted_account = env.get_account(1);
    env.set_caller(unlisted_account);

    assert_eq!(
        contract.try_mint(
            unlisted_account,
            TEST_PRETTY_721_META_DATA.to_string(),
            Maybe::None
        ),
        Err(CEP78Error::InvalidHolderMode.into())
    );
}

#[test]
#[ignore = "ACL package mode package mode is switched on by default and can't be switched off"]
fn should_disallow_contract_from_whitelisted_package_to_mint_without_acl_package_mode() {}

#[test]
#[ignore = "ACL package mode package mode is switched on by default and can't be switched off"]
fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode() {}

#[test]
#[ignore = "ACL package mode package mode is switched on by default and can't be switched off"]
fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode_after_contract_upgrade(
) {
}

// Update
#[test]
#[ignore = "Deprecated arg contract whitelist is not used in Odra"]
fn should_be_able_to_update_whitelist_for_minting_with_deprecated_arg_contract_whitelist() {}

#[test]
fn should_be_able_to_update_whitelist_for_minting() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);
    let contract_whitelist = vec![];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Unlocked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = CEP78HostRef::deploy(&env, args);

    assert!(
        !contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    assert_eq!(
        minting_contract.try_mint_for(
            contract.address(),
            env.get_account(0),
            TEST_PRETTY_721_META_DATA.to_string(),
        ),
        Err(CEP78Error::UnlistedContractHash.into()),
    );

    contract.set_variables(
        Maybe::None,
        Maybe::Some(vec![minting_contract.address().clone()]),
        Maybe::None
    );

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    assert!(minting_contract
        .try_mint_for(
            contract.address(),
            env.get_account(0),
            TEST_PRETTY_721_META_DATA.to_string(),
        )
        .is_ok());
}

// Upgrade
#[test]
#[ignore = "Odra implements v1.5.1, so this test is not applicable"]
fn should_upgrade_from_named_keys_to_dict_and_acl_minting_mode() {}
