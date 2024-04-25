use odra::{args::Maybe, host::Deployer};

use crate::cep78::{
    modalities::{
        NFTHolderMode, NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode
    },
    tests::{default_args_builder, utils},
    token::Cep78HostRef
};

#[test]
#[ignore = "Reverse lookup is not implemented yet"]
fn mint_cost_should_remain_stable() {
    let env = odra_test::env();
    let args = default_args_builder()
        .holder_mode(NFTHolderMode::Contracts)
        .whitelist_mode(WhitelistMode::Unlocked)
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .build();

    let mut _contract = Cep78HostRef::deploy(&env, args);
}

#[test]
fn transfer_costs_should_remain_stable() {
    let env = odra_test::env();
    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::Complete)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    let mut contract = Cep78HostRef::deploy(&env, args);
    let token_owner = env.get_account(0);
    contract.register_owner(Maybe::Some(token_owner));
    let receiver = env.get_account(1);

    for _ in 0..3 {
        contract.mint(token_owner, "".to_string(), Maybe::None);
    }

    contract.register_owner(Maybe::Some(receiver));
    contract.transfer(Maybe::Some(0u64), Maybe::None, token_owner, receiver);
    contract.transfer(Maybe::Some(1u64), Maybe::None, token_owner, receiver);
    contract.transfer(Maybe::Some(2u64), Maybe::None, token_owner, receiver);

    // We check only the second and third gas costs as the first transfer cost
    // has the additional gas of allocating a whole new page. Thus we ensure
    // that costs once a page has been allocated remain stable.
    let costs = utils::get_gas_cost_of(&env, "transfer");
    assert_eq!(costs.get(1), costs.get(2));
}

fn should_cost_less_when_installing_without_reverse_lookup(reporting: OwnerReverseLookupMode) {
    let env = odra_test::env();

    let args = default_args_builder()
        .owner_reverse_lookup_mode(OwnerReverseLookupMode::NoLookUp)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    Cep78HostRef::deploy(&env, args);
    // let page_dictionary_lookup = builder.query(None, no_lookup_hash, &["page_0".to_string()]);
    // assert!(page_dictionary_lookup.is_err());

    let args = default_args_builder()
        .owner_reverse_lookup_mode(reporting)
        .nft_metadata_kind(NFTMetadataKind::Raw)
        .ownership_mode(OwnershipMode::Transferable)
        .build();
    Cep78HostRef::deploy(&env, args);

    // let page_dictionary_lookup = builder.query(None, reverse_lookup_hash, &["page_0".to_string()]);
    // assert!(page_dictionary_lookup.is_ok());

    let costs = utils::get_deploy_gas_cost(&env);
    if let Some(no_lookup_gas_cost) = costs.first() {
        if let Some(reverse_lookup_gas_cost) = costs.get(1) {
            assert!(no_lookup_gas_cost < reverse_lookup_gas_cost);
        }
    }
}

#[test]
#[ignore = "Reverse lookup is not implemented yet"]
fn should_cost_less_when_installing_without_reverse_lookup_but_complete() {
    should_cost_less_when_installing_without_reverse_lookup(OwnerReverseLookupMode::Complete);
}

#[test]
#[ignore = "Reverse lookup is not implemented yet"]
fn should_cost_less_when_installing_without_reverse_lookup_but_transfer_only() {
    should_cost_less_when_installing_without_reverse_lookup(OwnerReverseLookupMode::TransfersOnly);
}
