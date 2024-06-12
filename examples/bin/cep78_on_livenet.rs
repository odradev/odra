//! Deploys a CEP-78 contract, mints an nft token and transfers it to another address.
use odra::args::Maybe;
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::Address;
use odra_modules::cep78::modalities::{
    EventsMode, MetadataMutability, NFTIdentifierMode, NFTKind, NFTMetadataKind, OwnershipMode
};
use odra_modules::cep78::token::{TestCep78HostRef, TestCep78InitArgs};
use odra_modules::cep78::utils::InitArgsBuilder;

const CEP78_METADATA: &str = r#"{
    "name": "John Doe",
    "token_uri": "https://www.barfoo.com",
    "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
const CASPER_CONTRACT_ADDRESS: &str =
    "hash-d4b8fa492d55ac7a515c0c6043d72ba43c49cd120e7ba7eec8c0a330dedab3fb";
const ODRA_CONTRACT_ADDRESS: &str =
    "hash-3d35238431c5c6fa1d7df70d73bfc2efd5a03fd5af99ab8c7828a56b2f330274";
const RECIPIENT_ADDRESS: &str =
    "hash-7821386ecdda83ff100379a06558b69a675d5a170d1c5bf5fbe9fd35262d091f";

fn main() {
    let env = odra_casper_livenet_env::env();

    // Deploy new contract.
    let mut token = deploy_contract(&env);
    println!("Token address: {}", token.address().to_string());

    // Uncomment to load existing contract.
    // let mut token = load_contract(&env, CASPER_CONTRACT_ADDRESS);
    // println!("Token name: {}", token.get_collection_name());

    env.set_gas(3_000_000_000u64);
    let owner = env.caller();
    let recipient =
        Address::new(RECIPIENT_ADDRESS).expect("Should be a valid recipient address");
    // casper contract may return a result or not, so deserialization may fail and it's better to use `try_transfer`/`try_mint`/`try_burn` methods
    let _ = token.try_mint(owner, CEP78_METADATA.to_string(), Maybe::None);
    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
    let token_id = token.get_number_of_minted_tokens() - 1;
    let _ = token.try_transfer(Maybe::Some(token_id), Maybe::None, owner, recipient);

    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
}

/// Loads a Cep78 contract.
pub fn load_contract(env: &HostEnv, address: &str) -> TestCep78HostRef {
    let address = Address::new(address).expect("Should be a valid contract address");
    TestCep78HostRef::load(env, address)
}

/// Deploys a Cep78 contract.
pub fn deploy_contract(env: &HostEnv) -> TestCep78HostRef {
    let name: String = String::from("PlascoinCollection with CES");
    let symbol = String::from("CEP78-PLS-CES");
    let receipt_name = String::from("PlascoinReceipt");

    let init_args = InitArgsBuilder::default()
        .collection_name(name)
        .collection_symbol(symbol)
        .total_token_supply(1_000)
        .ownership_mode(OwnershipMode::Transferable)
        .nft_metadata_kind(NFTMetadataKind::CEP78)
        .identifier_mode(NFTIdentifierMode::Ordinal)
        .nft_kind(NFTKind::Digital)
        .metadata_mutability(MetadataMutability::Mutable)
        .receipt_name(receipt_name)
        .events_mode(EventsMode::CES)
        .build();

    env.set_gas(430_000_000_000u64);
    TestCep78HostRef::deploy(env, init_args)
}
