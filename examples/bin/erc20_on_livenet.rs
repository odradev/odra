use odra::client_env;
use odra::types::{casper_types::U256, Address};
use odra_modules::erc20::Erc20Deployer;
use std::str::FromStr;

fn main() {
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 10u8;
    let initial_supply: U256 = U256::from(10_000);

    let owner = client_env::caller();
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    client_env::set_gas(110_000_000_000u64);
    let mut token = Erc20Deployer::init(name, symbol, decimals, &Some(initial_supply));

    // Uncomment to use already deployed contract.
    // let address = "hash-a12760e3ece51e0f31aa6d5af39660f5ec61185ad61c7551c796cca4592b9498";
    // let address = Address::from_str(address).unwrap();
    // let mut token = Erc20Deployer::register(address);

    println!("Token name: {}", token.name());

    client_env::set_gas(3_000_000_000u64);
    token.transfer(&recipient, &U256::from(1000));

    println!("Owner's balance: {:?}", token.balance_of(&owner));
    println!("Recipient's balance: {:?}", token.balance_of(&recipient));
}
