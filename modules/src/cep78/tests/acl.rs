use odra::{
    args::Maybe,
    host::{Deployer, HostEnv, HostRef, NoArgs},
    prelude::*,
    Address
};

use crate::cep78::{
    error::CEP78Error,
    modalities::{
        MintingMode, NFTHolderMode, NFTMetadataKind, OwnershipMode,
        WhitelistMode
    },
    tests::utils::{InitArgsBuilder, TEST_PRETTY_721_META_DATA},
    token::CEP78HostRef
};

fn default_args_builder() -> InitArgsBuilder {
    InitArgsBuilder::default()
        .total_token_supply(100u64)
        .nft_metadata_kind(NFTMetadataKind::NFT721)
}

#[odra::module]
struct DummyContract;

#[odra::module]
impl DummyContract {}

#[odra::module]
struct TestContract;

#[odra::module]
impl TestContract {
    pub fn mint(
        &mut self,
        nft_contract_address: &Address,
        token_metadata: String
    ) -> (String, Address, String) {
        NftContractContractRef::new(self.env(), *nft_contract_address)
            .mint(self.env().self_address(), token_metadata)
    }
}

#[odra::external_contract]
trait NftContract {
    fn mint(&mut self, token_owner: Address, token_metadata: String) -> (String, Address, String);
}

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
#[ignore = "No need to implement a contract whitelist"]
fn should_not_install_with_minting_mode_not_acl_if_acl_whitelist_provided() {}

fn should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
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
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
        &env,
        NFTHolderMode::Accounts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
            &env,
    NFTHolderMode::Contracts,
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_in_public_mode_with_holder_mode(
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
    let mut contract = CEP78HostRef::deploy(&env, args);
    assert!(
        contract.is_whitelisted(minting_contract.address()),
        "acl whitelist is incorrectly set"
    );

    minting_contract.mint(
        contract.address(),
        TEST_PRETTY_721_META_DATA.to_string()
    );

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(&actual_token_owner, minting_contract.address())
}

#[test]
fn should_disallow_unlisted_contract_from_minting() {
    let env = odra_test::env();

    let mut minting_contract = TestContractHostRef::deploy(&env, NoArgs);

    let contract_whitelist = vec![
        env.get_account(1),
        env.get_account(2),
        env.get_account(3),
    ];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Locked)
        .ownership_mode(OwnershipMode::Minter)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = CEP78HostRef::deploy(&env, args);

    assert_eq!(
        minting_contract.try_mint(
            contract.address(),
            TEST_PRETTY_721_META_DATA.to_string(),
        ),
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

    minting_contract.mint(
        contract.address(), 
        TEST_PRETTY_721_META_DATA.to_string()
    );

    let token_id = 0u64;
    let actual_token_owner = contract.owner_of(Maybe::Some(token_id), Maybe::None);
    assert_eq!(&actual_token_owner, minting_contract.address());

    assert!(
        contract.is_whitelisted(&account_user_1),
        "acl whitelist is incorrectly set"
    );
    env.set_caller(account_user_1);
    contract.mint(account_user_1, TEST_PRETTY_721_META_DATA.to_string(), Maybe::None);

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
        minting_contract.try_mint(
            contract.address(),
            TEST_PRETTY_721_META_DATA.to_string(),
        ),
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
    let mixed_whitelist = vec![
        minting_contract.address().clone(),
        listed_account,
    ];

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

// #[test]
// fn should_disallow_listed_account_from_minting_with_nftholder_contract() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);
//     let mixed_whitelist = vec![
//         Key::from(minting_contract_hash),
//         Key::from(*DEFAULT_ACCOUNT_ADDR),
//     ];

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Locked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(mixed_whitelist)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_hash = get_nft_contract_hash(&builder);
//     let nft_contract_key: Key = nft_contract_hash.into();

//     let is_whitelisted_account = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &DEFAULT_ACCOUNT_ADDR.to_string(),
//     );

//     assert!(is_whitelisted_account, "acl whitelist is incorrectly set");

//     let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER =>  Key::Account(account_user_1),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false
//     };

//     let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
//         account_user_1,
//         nft_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder.exec(mint_session_call).expect_failure();

//     let error = builder.get_error().expect("should have an error");
//     assert_expected_error(error, 76, "InvalidHolderMode(76) must have been raised");
// }

// #[test]
// fn should_disallow_contract_from_whitelisted_package_to_mint_without_acl_package_mode() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);
//     let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

//     let contract_whitelist = vec![Key::from(minting_contract_package_hash)];

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Locked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(contract_whitelist)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

//     let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_package_hash.to_string(),
//     );

//     assert!(
//         is_whitelisted_contract_package,
//         "acl whitelist is incorrectly set"
//     );

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false
//     };

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder.exec(mint_via_contract_call).expect_failure();

//     let error = builder.get_error().expect("should have an error");
//     assert_expected_error(
//         error,
//         81,
//         "Unlisted ContractHash from whitelisted ContractPackageHash can not mint without ACL package mode",
//     );
// }

// #[test]
// fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);
//     let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

//     let contract_whitelist = vec![Key::from(minting_contract_package_hash)];
//     let acl_package_mode = true;

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Locked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(contract_whitelist)
//         .with_acl_package_mode(acl_package_mode)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

//     let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_package_hash.to_string(),
//     );

//     assert!(
//         is_whitelisted_contract_package,
//         "acl whitelist is incorrectly set"
//     );

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false
//     };

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder
//         .exec(mint_via_contract_call)
//         .expect_success()
//         .commit();

//     let token_id = 0u64;

//     let actual_token_owner: Key = get_dictionary_value_from_key(
//         &builder,
//         &nft_contract_key,
//         TOKEN_OWNERS,
//         &token_id.to_string(),
//     );

//     let minting_contract_key: Key = minting_contract_hash.into();

//     assert_eq!(actual_token_owner, minting_contract_key)
// }

// #[test]
// fn should_allow_contract_from_whitelisted_package_to_mint_with_acl_package_mode_after_contract_upgrade(
// ) {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);
//     let minting_contract_package_hash = get_minting_contract_package_hash(&builder);

//     let contract_whitelist = vec![Key::from(minting_contract_package_hash)];
//     let acl_package_mode = true;

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Locked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(contract_whitelist)
//         .with_acl_package_mode(acl_package_mode)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

//     let is_whitelisted_contract_package = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_package_hash.to_string(),
//     );

//     assert!(
//         is_whitelisted_contract_package,
//         "acl whitelist is incorrectly set"
//     );

//     let version_minting_contract = support::query_stored_value::<u32>(
//         &builder,
//         Key::Account(*DEFAULT_ACCOUNT_ADDR),
//         vec![MINTING_CONTRACT_VERSION.to_string()],
//     );

//     assert_eq!(version_minting_contract, 1u32);

//     let upgrade_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder.exec(upgrade_request).expect_success().commit();

//     let version_minting_contract = support::query_stored_value::<u32>(
//         &builder,
//         Key::Account(*DEFAULT_ACCOUNT_ADDR),
//         vec![MINTING_CONTRACT_VERSION.to_string()],
//     );

//     assert_eq!(version_minting_contract, 2u32);

//     let minting_upgraded_contract_hash = get_minting_contract_hash(&builder);
//     assert_ne!(minting_contract_hash, minting_upgraded_contract_hash);

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER => Key::from(minting_contract_hash),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false
//     };

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_upgraded_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder
//         .exec(mint_via_contract_call)
//         .expect_success()
//         .commit();

//     let token_id = 0u64;

//     let actual_token_owner: Key = get_dictionary_value_from_key(
//         &builder,
//         &nft_contract_key,
//         TOKEN_OWNERS,
//         &token_id.to_string(),
//     );

//     let minting_contract_key: Key = minting_upgraded_contract_hash.into();

//     assert_eq!(actual_token_owner, minting_contract_key)
// }

// // Update

// #[test]
// fn should_be_able_to_update_whitelist_for_minting_with_deprecated_arg_contract_whitelist() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);

//     let contract_whitelist = vec![];

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Unlocked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(contract_whitelist)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_hash = get_nft_contract_hash(&builder);
//     let nft_contract_key: Key = nft_contract_hash.into();

//     let seed_uref = *builder
//         .query(None, nft_contract_key, &[])
//         .expect("must have nft contract")
//         .as_contract()
//         .expect("must convert contract")
//         .named_keys()
//         .get(ACL_WHITELIST)
//         .expect("must have key")
//         .as_uref()
//         .expect("must convert to seed uref");

//     let is_whitelisted_account =
//         builder.query_dictionary_item(None, seed_uref, &minting_contract_hash.to_string());

//     assert!(
//         is_whitelisted_account.is_err(),
//         "acl whitelist is incorrectly set"
//     );

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false,
//     };

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args.clone(),
//     )
//     .build();

//     builder.exec(mint_via_contract_call).expect_failure();

//     let error = builder.get_error().expect("should have an error");
//     assert_expected_error(
//         error,
//         81,
//         "Unlisted contract hash should not be permitted to mint",
//     );

//     let update_whitelist_request = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         nft_contract_hash,
//         ENTRY_POINT_SET_VARIABLES,
//         runtime_args! {
//             ARG_CONTRACT_WHITELIST => vec![minting_contract_hash]
//         },
//     )
//     .build();

//     builder
//         .exec(update_whitelist_request)
//         .expect_success()
//         .commit();

//     let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_hash.to_string(),
//     );

//     assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder
//         .exec(mint_via_contract_call)
//         .expect_success()
//         .commit();
// }

// #[test]
// fn should_be_able_to_update_whitelist_for_minting() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);

//     let contract_whitelist = vec![];

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
//         .with_total_token_supply(100u64)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Unlocked)
//         .with_ownership_mode(OwnershipMode::Minter)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_minting_mode(MintingMode::Acl)
//         .with_acl_whitelist(contract_whitelist)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_hash = get_nft_contract_hash(&builder);
//     let nft_contract_key: Key = nft_contract_hash.into();

//     let seed_uref = *builder
//         .query(None, nft_contract_key, &[])
//         .expect("must have nft contract")
//         .as_contract()
//         .expect("must convert contract")
//         .named_keys()
//         .get(ACL_WHITELIST)
//         .expect("must have key")
//         .as_uref()
//         .expect("must convert to seed uref");

//     let is_whitelisted_account =
//         builder.query_dictionary_item(None, seed_uref, &minting_contract_hash.to_string());

//     assert!(
//         is_whitelisted_account.is_err(),
//         "acl whitelist is incorrectly set"
//     );

//     let mint_runtime_args = runtime_args! {
//         ARG_NFT_CONTRACT_HASH => nft_contract_key,
//         ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
//         ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
//         ARG_REVERSE_LOOKUP => false,
//     };

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args.clone(),
//     )
//     .build();

//     builder.exec(mint_via_contract_call).expect_failure();

//     let error = builder.get_error().expect("should have an error");
//     assert_expected_error(
//         error,
//         81,
//         "Unlisted contract hash should not be permitted to mint",
//     );

//     let update_whitelist_request = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         nft_contract_hash,
//         ENTRY_POINT_SET_VARIABLES,
//         runtime_args! {
//             ARG_ACL_WHITELIST => vec![Key::from(minting_contract_hash)]
//         },
//     )
//     .build();

//     builder
//         .exec(update_whitelist_request)
//         .expect_success()
//         .commit();

//     let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_hash.to_string(),
//     );

//     assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

//     let mint_via_contract_call = ExecuteRequestBuilder::contract_call_by_hash(
//         *DEFAULT_ACCOUNT_ADDR,
//         minting_contract_hash,
//         ENTRY_POINT_MINT,
//         mint_runtime_args,
//     )
//     .build();

//     builder
//         .exec(mint_via_contract_call)
//         .expect_success()
//         .commit();
// }

// // Upgrade

// #[test]
// fn should_upgrade_from_named_keys_to_dict_and_acl_minting_mode() {
//     let mut builder = InMemoryWasmTestBuilder::default();
//     builder
//         .run_genesis(&PRODUCTION_RUN_GENESIS_REQUEST)
//         .commit();

//     let minting_contract_install_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         MINTING_CONTRACT_WASM,
//         runtime_args! {},
//     )
//     .build();

//     builder
//         .exec(minting_contract_install_request)
//         .expect_success()
//         .commit();

//     let minting_contract_hash = get_minting_contract_hash(&builder);
//     let contract_whitelist = vec![minting_contract_hash];

//     let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
//         .with_collection_name(NFT_TEST_COLLECTION.to_string())
//         .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
//         .with_total_token_supply(1000u64)
//         .with_minting_mode(MintingMode::Installer)
//         .with_holder_mode(NFTHolderMode::Contracts)
//         .with_whitelist_mode(WhitelistMode::Locked)
//         .with_ownership_mode(OwnershipMode::Transferable)
//         .with_nft_metadata_kind(NFTMetadataKind::Raw)
//         .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
//         .with_contract_whitelist(contract_whitelist)
//         .build();

//     builder.exec(install_request).expect_success().commit();

//     let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
//     let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

//     let minting_mode = support::query_stored_value::<u8>(
//         &builder,
//         nft_contract_key_1_0_0,
//         vec![ARG_MINTING_MODE.to_string()],
//     );

//     assert_eq!(
//         minting_mode,
//         MintingMode::Installer as u8,
//         "minting mode should be set to public"
//     );

//     let upgrade_request = ExecuteRequestBuilder::standard(
//         *DEFAULT_ACCOUNT_ADDR,
//         NFT_CONTRACT_WASM,
//         runtime_args! {
//             ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
//             ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
//             ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
//         },
//     )
//     .build();

//     builder.exec(upgrade_request).expect_success().commit();

//     let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

//     let is_updated_acl_whitelist = get_dictionary_value_from_key::<bool>(
//         &builder,
//         &nft_contract_key,
//         ACL_WHITELIST,
//         &minting_contract_hash.to_string(),
//     );

//     assert!(is_updated_acl_whitelist, "acl whitelist is incorrectly set");

//     let minting_mode = support::query_stored_value::<u8>(
//         &builder,
//         nft_contract_key,
//         vec![ARG_MINTING_MODE.to_string()],
//     );

//     assert_eq!(
//         minting_mode,
//         MintingMode::Acl as u8,
//         "minting mode should be set to acl"
//     );
// }
