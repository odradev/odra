//! Deploys a CEP-18 contract and transfers some tokens to another address.
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::Address;
use odra_modules::cep18_token::{Cep18HostRef, Cep18InitArgs};
use std::str::FromStr;

fn main() {
    let env = odra_casper_livenet_env::env();

    let owner = env.caller();
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // Deploy new contract.
    let mut token = deploy_cep18(&env);
    println!("Token address: {}", token.address().to_string());

    // Uncomment to load existing contract.
    // let mut token = load_erc20(&env);

    // println!("Token name: {}", token.name());

    env.set_gas(3_000_000_000u64);
    token.transfer(&recipient, &U256::from(1000));
    
    token.approve(&recipient, &U256::from(3500));
    //
    // println!("Owner's balance: {:?}", token.balance_of(&owner));
    // println!("Recipient's balance: {:?}", token.balance_of(&recipient));
}

/// Loads an ERC20 contract.
fn _load_cep18(env: &HostEnv) -> Cep18HostRef {
    let address = "hash-d26fcbd2106e37be975d2045c580334a6d7b9d0a241c2358a4db970dfd516945";
    let address = Address::from_str(address).unwrap();
    Cep18HostRef::load(env, address)
}

/// Deploys an ERC20 contract.
pub fn deploy_cep18(env: &HostEnv) -> Cep18HostRef {
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 2u8;
    let initial_supply = U256::from(10_000);

let init_args = Cep18InitArgs {
        name,
        symbol,
        decimals,
        initial_supply,
        minter_list: vec![],
        admin_list: vec![env.caller()],
        modality: Some(1)
    };

    env.set_gas(200_000_000_000u64);
    Cep18HostRef::deploy(env, init_args)
}
