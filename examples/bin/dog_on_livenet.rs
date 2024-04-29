//! Deploys a simple Dog contract and barks.
use std::str::FromStr;

use odra::host::{Deployer, HostEnv, HostRefLoader};
use odra::Address;
use odra_examples::features::storage::mapping::{DogContract2HostRef, DogContract2InitArgs};

fn main() {
    let env = odra_casper_livenet_env::env();

    // Deploy new contract.
    // let mut dog = _deploy_dog(&env);
    // println!("Token address: {}", dog.address().to_string());

    // Uncomment to load existing contract.
    let mut dog = load_dog(&env);

    assert_eq!(dog.name(), "Mantus".to_string());
    env.set_gas(1_000_000_000u64);
    let visits = dog.visits(&"Grzesiek".to_string());

    dog.visit(&"Grzesiek".to_string());
    assert_eq!(dog.visits(&"Grzesiek".to_string()), visits + 1);
}

/// Loads a Dog contract.
fn load_dog(env: &HostEnv) -> DogContract2HostRef {
    let address = "hash-c2e743f91d242c599fc5b54fec169c029db7e9e90817d4714af7e293a009a8f9";
    let address = Address::from_str(address).unwrap();
    DogContract2HostRef::load(env, address)
}

/// Deploys a Dog contract.
pub fn _deploy_dog(env: &HostEnv) -> DogContract2HostRef {
    let init_args = DogContract2InitArgs {
        name: "Mantus".to_string()
    };

    env.set_gas(300_000_000_000u64);

    DogContract2HostRef::deploy(env, init_args)
}
