use odra::host::{Deployer, HostRef, NoArgs};

use crate::cep78::{
    error::CEP78Error,
    modalities::{
        MintingMode, NFTHolderMode, OwnerReverseLookupMode, OwnershipMode, WhitelistMode
    },
    tests::{default_args_builder, COLLECTION_NAME, COLLECTION_SYMBOL},
    token::TestCep78,
    utils::MockDummyContract
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
    let contract = TestCep78::deploy(&env, args);

    assert_eq!(&contract.get_collection_name(), COLLECTION_NAME);
    assert_eq!(&contract.get_collection_symbol(), COLLECTION_SYMBOL);
    assert_eq!(contract.get_total_supply(), 1u64);
    assert!(contract.is_minting_allowed());
    assert_eq!(contract.get_minting_mode(), MintingMode::Installer);
    assert_eq!(contract.get_number_of_minted_tokens(), 0u64);
}

#[test]
#[ignore = "Not applicable in Odra, init is not allowed after installation by design"]
fn should_only_allow_init_during_installation_session() {}

#[test]
fn should_install_with_allow_minting_set_to_false() {
    let env = odra_test::env();

    let args = default_args_builder().allow_minting(false).build();
    let contract = TestCep78::deploy(&env, args);
    assert!(!contract.is_minting_allowed());
}

#[test]
#[ignore = "Odra interface does not allow to pass a wrong type"]
fn should_reject_invalid_collection_name() {}

#[test]
#[ignore = "Odra interface does not allow to pass a wrong type"]
fn should_reject_invalid_collection_symbol() {}

#[test]
#[ignore = "Odra interface does not allow to pass a wrong type"]
fn should_reject_non_numerical_total_token_supply_value() {}

#[test]
fn should_install_with_contract_holder_mode() {
    let env = odra_test::env();
    let whitelisted_contract = MockDummyContract::deploy(&env, NoArgs);
    let contract_whitelist = vec![*whitelisted_contract.address()];
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Unlocked)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::NoLookUp)
        .minting_mode(MintingMode::Acl)
        .acl_white_list(contract_whitelist)
        .build();
    let contract = TestCep78::deploy(&env, args);

    assert_eq!(
        contract.get_holder_mode(),
        NFTHolderMode::Contracts,
        "holder mode is not set to contracts"
    );

    assert_eq!(
        contract.get_whitelist_mode(),
        WhitelistMode::Unlocked,
        "whitelist mode is not set to unlocked"
    );

    let is_whitelisted_account = contract.is_whitelisted(whitelisted_contract.address());
    assert!(is_whitelisted_account, "acl whitelist is incorrectly set");
}

fn should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
    nft_holder_mode: NFTHolderMode
) {
    let env = odra_test::env();
    let args = default_args_builder()
        .holder_mode(nft_holder_mode)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::NoLookUp)
        .whitelist_mode(WhitelistMode::Locked)
        .minting_mode(MintingMode::Acl)
        .build();

    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::EmptyACLWhitelist.into()),
        "should fail execution since whitelist mode is locked and the provided whitelist is empty",
    );
}

#[test]
fn should_disallow_installation_of_contract_with_empty_locked_whitelist() {
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Accounts
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Contracts
    );
    should_disallow_installation_of_contract_with_empty_locked_whitelist_with_holder_mode(
        NFTHolderMode::Mixed
    );
}

#[test]
fn should_disallow_installation_with_zero_issuance() {
    let env = odra_test::env();
    let args = default_args_builder().total_token_supply(0).build();
    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::CannotInstallWithZeroSupply.into()),
        "cannot install when issuance is equal 0",
    );
}

#[test]
fn should_disallow_installation_with_supply_exceeding_hard_cap() {
    let env = odra_test::env();
    let args = default_args_builder()
        .total_token_supply(1_000_001u64)
        .build();
    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::ExceededMaxTotalSupply.into()),
        "cannot install when issuance is more than 1_000_000",
    );
}

#[test]
fn should_prevent_installation_with_ownership_and_minting_modality_conflict() {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Installer)
        .ownership_mode(OwnershipMode::Minter)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .build();
    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::InvalidReportingMode.into()),
        "cannot install when Ownership::Minter and MintingMode::Installer",
    );
}

#[test]
fn should_prevent_installation_with_ownership_minter_and_owner_reverse_lookup_mode_transfer_only() {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Installer)
        .ownership_mode(OwnershipMode::Minter)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();
    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::OwnerReverseLookupModeNotTransferable.into()),
        "cannot install when Ownership::Minter and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_prevent_installation_with_ownership_assigned_and_owner_reverse_lookup_mode_transfer_only()
{
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Installer)
        .ownership_mode(OwnershipMode::Assigned)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();
    assert_eq!(
        TestCep78::try_deploy(&env, args).err(),
        Some(CEP78Error::OwnerReverseLookupModeNotTransferable.into()),
        "cannot install when Ownership::Minter and OwnerReverseLookupMode::TransfersOnly",
    );
}

#[test]
fn should_allow_installation_with_ownership_transferable_and_owner_reverse_lookup_mode_transfer_only(
) {
    let env = odra_test::env();
    let args = default_args_builder()
        .minting_mode(MintingMode::Installer)
        .ownership_mode(OwnershipMode::Transferable)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::TransfersOnly)
        .build();
    assert_eq!(TestCep78::try_deploy(&env, args).err(), None);
}
