use odra::casper_types::U256;
use odra::{Address, HostEnv};
use odra_modules::erc20::{Erc20Deployer, Erc20HostRef};
use std::str::FromStr;

fn main() {
    let env = odra_casper_livenet_env::env();

    let owner = env.caller();
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // Deploy new contract.
    let mut token = deploy_new(&env);
    println!("Token address: {}", token.address().to_string());

    // Uncomment to load existing contract.
    // let mut token = load(&env);

    println!("Token name: {}", token.name());

    env.set_gas(3_000_000_000u64);
    token.transfer(recipient, U256::from(1000));

    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
}

fn deploy_new(env: &HostEnv) -> Erc20HostRef {
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 10u8;
    let initial_supply: U256 = U256::from(10_000);

    env.set_gas(100_000_000_000u64);
    Erc20Deployer::init(env, name, symbol, decimals, Some(initial_supply))
}

fn _load(env: &HostEnv) -> Erc20HostRef {
    let address = "hash-d26fcbd2106e37be975d2045c580334a6d7b9d0a241c2358a4db970dfd516945";
    let address = Address::from_str(address).unwrap();
    Erc20Deployer::load(env, address)
}
