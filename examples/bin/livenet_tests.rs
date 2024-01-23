use odra::{Address, HostEnv};
use odra_examples::features::livenet::{LivenetContractDeployer, LivenetContractHostRef};
use odra_modules::access::events::OwnershipTransferred;
use odra_modules::erc20::Erc20Deployer;
use std::str::FromStr;

fn main() {
    let env = odra_casper_livenet_env::livenet_env();

    let owner = env.caller();

    // Contract can be deployed
    env.set_gas(30_000_000_000u64);
    let contract = deploy_new(&env);
    println!("Contract address: {}", contract.address().to_string());

    // Contract can be loaded
    let mut contract = load(&env, *contract.address());

    // Uncomment to load existing contract
    // let address = Address::from_str("hash-bd80e12b6d189e3b492932fc08e1342b540555c60e299ea3563d81ad700997e0").unwrap();
    // let mut contract = load(&env, address);

    // Set gas will be used for all subsequent calls
    env.set_gas(1_000_000_000u64);

    // There are three ways contract endpoints can be called in Livenet environment:
    // 1. If the endpoint is mutable and does not return anything, it can be called directly:
    contract.push_on_stack(1);

    // 2. If the endpoint is mutable and returns something, it can be called through the proxy:
    let value = contract.pop_from_stack();
    assert_eq!(value, 1);

    // 3. If the endpoint is immutable, it can be called locally, querying only storage from livenet:
    assert_eq!(contract.owner(), owner);

    // By querying livenet storage
    // - we can also test the events
    assert_eq!(env.events_count(contract.address()), 1);

    let event: OwnershipTransferred = env.get_event(contract.address(), 0).unwrap();
    assert_eq!(event.new_owner, Some(owner));

    // - we can test immutable crosscalls without deploying
    assert_eq!(contract.immutable_cross_call(), 10_000.into());

    // - mutable crosscalls will require a deploy
    let pre_call_balance = Erc20Deployer::load(&env, erc20_address()).balance_of(env.caller());
    contract.mutable_cross_call();
    let post_call_balance = Erc20Deployer::load(&env, erc20_address()).balance_of(env.caller());
    assert_eq!(post_call_balance, pre_call_balance + 1);

    // We can change the caller
}

fn deploy_new(env: &HostEnv) -> LivenetContractHostRef {
    env.set_gas(100_000_000_000u64);
    let livenet_contract = LivenetContractDeployer::init(env, erc20_address());
    Erc20Deployer::load(env, erc20_address()).transfer(*livenet_contract.address(), 1000.into());
    livenet_contract
}

fn erc20_address() -> Address {
    // Following contract is deployed on integration livenet, change it to address from erc20_on_livenet example
    Address::from_str("hash-d26fcbd2106e37be975d2045c580334a6d7b9d0a241c2358a4db970dfd516945")
        .unwrap()
}

fn load(env: &HostEnv, address: Address) -> LivenetContractHostRef {
    LivenetContractDeployer::load(env, address)
}
