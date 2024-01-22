use std::str::FromStr;
use odra::{Address, HostEnv};
use odra_examples::features::livenet::{LivenetContractDeployer, LivenetContractHostRef};

fn main() {
    let env = odra_casper_livenet_env::livenet_env();

    let owner = env.caller();

    // Contract can be deployed
    // env.set_gas(30_000_000_000u64);
    // let contract = deploy_new(&env);
    // println!("Contract address: {}", contract.address().to_string());

    // Contract can be loaded
    // let mut contract = load(&env, *contract.address());

    let address = Address::from_str("hash-b2cf71fcbf69eea9e8bfc7b76fb388ccfdc7233fbe1358fbc0959ef14511d756").unwrap();
    let mut contract = load(&env, address);

    // Set gas will be used for all subsequent calls
    env.set_gas(1_000_000_000u64);

    // There are three ways contract endpoints can be called in Livenet environment:
    // 1. If the endpoint is mutable and does not return anything, it can be called directly:
    // contract.push_on_stack(1);

    // 2. If the endpoint is mutable and returns something, it can be called through the proxy:
    // let value = contract.pop_from_stack();
    // assert_eq!(value, 1);

    // 3. If the endpoint is immutable, it can be called locally, querying only storage from livenet:
    // assert_eq!(contract.owner(), owner);

    // By querying livenet storage
    // - we can also test the events
    assert_eq!(env.events_count(contract.address()), 1);

    // - we can test immutable crosscalls

    // - mutable crosscalls will require a deploy

    // We can change the caller
}

fn deploy_new(env: &HostEnv) -> LivenetContractHostRef {
    env.set_gas(100_000_000_000u64);
    LivenetContractDeployer::init(env)
}

fn load(env: &HostEnv, address: Address) -> LivenetContractHostRef {
    LivenetContractDeployer::load(env, address)
}
