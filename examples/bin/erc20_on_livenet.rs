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
    dbg!(owner);
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // env.set_gas(100_000_000_000u64);
    // let mut token = Erc20Deployer::init(&env, name, symbol, decimals, Some(initial_supply));
    //
    // Uncomment to use already deployed contract.
    let address = "hash-c0eb363adc62699b2e2bbf5d2c2df4c3216b36df2e65b66ae394e824076de2de";
    let address = Address::from_str(address).unwrap();
    let mut token = Erc20HostRef::new(address, env);

    println!("Token name: {}", token.name());

    // env.set_gas(3_000_000_000u64);
    // token.transfer(recipient, U256::from(1000));
    //
    // println!("Owner's balance: {:?}", token.balance_of(owner));
    // println!("Recipient's balance: {:?}", token.balance_of(recipient));
}
