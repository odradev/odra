use odra::{
    args::Maybe,
    host::{Deployer, HostEnv, HostRef, NoArgs},
    prelude::*
};

use crate::cep78::{
    error::CEP78Error,
    modalities::{MintingMode, NFTHolderMode, OwnershipMode, WhitelistMode},
    tests::utils::TEST_PRETTY_721_META_DATA,
    token::TestCep78,
    utils::{MockCep78Operator, MockDummyContract}
};

use super::default_args_builder;

#[test]
fn should_install_with_acl_whitelist() {
    let env = odra_test::env();
    let test_contract_address = MockCep78Operator::deploy(&env, NoArgs);
    let contract_whitelist = vec![*test_contract_address.address()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = TestCep78::deploy(&env, args);

    assert_eq!(WhitelistMode::Locked, contract.get_whitelist_mode());
    let is_whitelisted_contract = contract.is_whitelisted(test_contract_address.address());
    assert!(is_whitelisted_contract, "acl whitelist is incorrectly set");
}

#[test]
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

    let init_result = TestCep78::try_deploy(&env, args);
    assert_eq!(
        init_result.err(),
        Some(CEP78Error::InvalidMintingMode.into()),
        "should disallow installing with minting mode not acl if acl whitelist provided"
    );
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

    assert!(TestCep78::try_deploy(env, args).is_ok());
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

#[test]
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

    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::InvalidMintingMode.into())
    );
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
    let mut contract = TestCep78::deploy(&env, args);

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
    let mut contract = TestCep78::deploy(&env, args);

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

    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);

    let contract_whitelist = vec![*minting_contract.address()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = TestCep78::deploy(&env, args);
    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );
    minting_contract.set_address(contract.address());
    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), false);

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(&actual_token_owner, minting_contract.address())
}

#[test]
fn should_disallow_unlisted_contract_from_minting() {
    let env = odra_test::env();

    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);

    let contract_whitelist = vec![env.get_account(1), env.get_account(2), env.get_account(3)];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert_eq!(
        minting_contract.try_mint(TEST_PRETTY_721_META_DATA.to_string(), false),
        Err(CEP78Error::UnlistedContractHash.into()),
        "Unlisted account hash should not be permitted to mint"
    );
}

#[test]
fn should_allow_mixed_account_contract_to_mint() {
    let env = odra_test::env();

    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let account_user_1 = env.get_account(1);
    let mixed_whitelist = vec![*minting_contract.address(), account_user_1];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(TEST_PRETTY_721_META_DATA.to_string(), false);

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

    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let account_user_1 = env.get_account(1);
    let mixed_whitelist = vec![
        *MockDummyContract::deploy(&env, NoArgs).address(),
        account_user_1,
    ];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert_eq!(
        minting_contract.try_mint(TEST_PRETTY_721_META_DATA.to_string(), false),
        Err(CEP78Error::UnlistedContractHash.into()),
        "Unlisted contract should not be permitted to mint"
    );
}

#[test]
fn should_disallow_unlisted_account_from_minting_with_mixed_account_contract() {
    let env = odra_test::env();

    let minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let listed_account = env.get_account(0);
    let unlisted_account = env.get_account(1);
    let mixed_whitelist = vec![*minting_contract.address(), listed_account];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Mixed)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
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

    let minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let listed_account = env.get_account(0);

    let mixed_whitelist = vec![*minting_contract.address(), listed_account];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(mixed_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);

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
fn should_be_able_to_update_whitelist_for_minting() {
    let env = odra_test::env();

    let mut minting_contract = MockCep78Operator::deploy(&env, NoArgs);
    let contract_whitelist = vec![];

    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Unlocked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let mut contract = TestCep78::deploy(&env, args);
    minting_contract.set_address(contract.address());

    assert!(
        !contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    assert_eq!(
        minting_contract.try_mint_for(env.get_account(0), TEST_PRETTY_721_META_DATA.to_string(),),
        Err(CEP78Error::UnlistedContractHash.into()),
    );

    contract.set_variables(
        Maybe::None,
        Maybe::Some(vec![*minting_contract.address()]),
        Maybe::None
    );

    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    assert!(minting_contract
        .try_mint_for(env.get_account(0), TEST_PRETTY_721_META_DATA.to_string(),)
        .is_ok());
}
