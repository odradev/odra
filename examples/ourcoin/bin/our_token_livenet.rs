//! Deploys a new OurToken contract on the Casper livenet and mints some tokens for the tutorial
//! creator.
use std::str::FromStr;

use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::Address;
use ourcoin::token::{OurTokenHostRef, OurTokenInitArgs};

fn main() {
    // Load the Casper livenet environment.
    let env = odra_casper_livenet_env::env();

    // Caller is the deployer and the owner of the private key.
    let owner = env.caller();
    // Just some random address...
    let recipient = "hash-48bd92253a1370d1d913c56800296145547a243d13ff4f059ba4b985b1e94c26";
    let recipient = Address::from_str(recipient).unwrap();

    // Deploy new contract.
    let mut token = deploy_our_token(&env);
    println!("Token address: {}", token.address().to_string());

    // Propose minting new tokens.
    env.set_gas(1_000_000_000u64);
    token.propose_new_mint(recipient, U256::from(1_000));

    // Vote, we are the only voter.
    env.set_gas(1_000_000_000u64);
    token.vote(true, U256::from(1_000));

    // Let's advance the block time by 11 minutes, as
    // we set the voting time to 10 minutes.
    // OH NO! It is the Livenet, so we need to wait real time...
    // Hopefully you are not in a hurry.
    env.advance_block_time(11 * 60 * 1000);

    // Tally the votes.
    env.set_gas(1_500_000_000u64);
    token.tally();

    // Check the balances.
    println!("Owner's balance: {:?}", token.balance_of(&owner));
    println!(
        "Tutorial creator's balance: {:?}",
        token.balance_of(&recipient)
    );
}

/// Loads a contract. Just in case you need to load an existing contract later...
fn _load_cep18(env: &HostEnv) -> OurTokenHostRef {
    let address = "hash-XXXXX";
    let address = Address::from_str(address).unwrap();
    OurTokenHostRef::load(env, address)
}

/// Deploys a contract.
pub fn deploy_our_token(env: &HostEnv) -> OurTokenHostRef {
    let name = String::from("OurToken");
    let symbol = String::from("OT");
    let decimals = 0;
    let initial_supply = U256::from(1_000);

    let init_args = OurTokenInitArgs {
        name,
        symbol,
        decimals,
        initial_supply
    };

    env.set_gas(300_000_000_000u64);
    OurTokenHostRef::deploy(env, init_args)
}
