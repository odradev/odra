//! Deploys a CEP-78 contract and transfers some tokens to another address.
use std::str::FromStr;

use odra::args::Maybe;
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::Address;
use odra_modules::cep78::modalities::{
    EventsMode, MetadataMutability, NFTIdentifierMode, NFTKind, NFTMetadataKind, OwnershipMode
};
use odra_modules::cep78::token::{Cep78HostRef, Cep78InitArgs};
use odra_modules::cep78::utils::InitArgsBuilder;

const CEP78_METADATA: &str = r#"{
    "name": "John Doe",
    "token_uri": "https://www.barfoo.com",
    "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;

fn main() {
    let env = odra_casper_livenet_env::env();

    let owner = env.caller();
    let recipient =
        Address::from_str("hash-7821386ecdda83ff100379a06558b69a675d5a170d1c5bf5fbe9fd35262d091f")
            .unwrap();

    // Deploy new contract.
    // let mut token = deploy_contract(&env);
    // println!("Token address: {}", token.address().to_string());

    // Uncomment to load existing contract.
    let mut token = _load_contract(&env);

    println!("Token name: {}", token.get_collection_name());

    env.set_gas(3_000_000_000u64);
    token.try_mint(owner, CEP78_METADATA.to_string(), Maybe::None);
    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
    let token_id = token.get_number_of_minted_tokens() - 1;
    token.try_transfer(Maybe::Some(token_id), Maybe::None, owner, recipient);

    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
}

/// Loads a Cep78 contract.
fn _load_contract(env: &HostEnv) -> Cep78HostRef {
    // casper-contract
    // let address = "hash-d4b8fa492d55ac7a515c0c6043d72ba43c49cd120e7ba7eec8c0a330dedab3fb";
    // odra-contract
    let address = "hash-3d35238431c5c6fa1d7df70d73bfc2efd5a03fd5af99ab8c7828a56b2f330274";
    let address = Address::from_str(address).unwrap();
    Cep78HostRef::load(env, address)
}

/// Deploys a Cep78 contract.
pub fn deploy_contract(env: &HostEnv) -> Cep78HostRef {
    let name: String = String::from("PlascoinCollection with CES");
    let symbol = String::from("CEP78-PLS-CES");

    let init_args = InitArgsBuilder::default()
        .collection_name(name)
        .collection_symbol(symbol)
        .total_token_supply(1_000)
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_kind(NFTKind::Digital)
        .metadata_mutability(MetadataMutability::Mutable)
        .receipt_name(String::from("PlascoinReceipt"))
        .events_mode(EventsMode::CES)
        .build();

    env.set_gas(400_000_000_000u64);
    Cep78HostRef::deploy(env, init_args)
}
