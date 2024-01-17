use std::str::FromStr;
use odra::{Address, U256};
use odra_modules::erc20::{Erc20Deployer, Erc20HostRef};

fn main() {
    let env = odra_casper_livenet_env::livenet_env();
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 10u8;
    let initial_supply: U256 = U256::from(10_000);

    let owner = env.caller();
    // dbg!(owner);
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // env.set_gas(100_000_000_000u64);
    // let mut token = Erc20Deployer::init(&env, name, symbol, decimals, Some(initial_supply));

    // Uncomment to use already deployed contract.
    let address = "hash-d26fcbd2106e37be975d2045c580334a6d7b9d0a241c2358a4db970dfd516945";
    let address = Address::from_str(address).unwrap();
    let mut token = Erc20Deployer::load(&env, address);
    // env.set_gas(1_000_000_000u64);
    // token.approve(owner, U256::from(1000));
    // let name = token.name();

    println!("Token name: {}", token.symbol());

    // env.set_gas(3_000_000_000u64);
    // token.transfer(recipient, U256::from(1000));
    //
    // println!("Owner's balance: {:?}", token.balance_of(owner));
    // println!("Recipient's balance: {:?}", token.balance_of(recipient));
}
