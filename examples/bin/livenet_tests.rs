//! This example demonstrates how to deploy and interact with a contract on the Livenet environment.
use std::time::Duration;

use odra::casper_types::{U256, U512};
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader, InstallConfig, NoArgs};
use odra::prelude::*;
use odra_examples::features::livenet::{
    LivenetContract, LivenetContractHostRef, LivenetContractInitArgs
};
use odra_examples::features::upgrade::{CounterV1, CounterV2, CounterV2UpgradeArgs};
use odra_modules::access::events::OwnershipTransferred;
use odra_modules::erc20::{Erc20, Erc20HostRef, Erc20InitArgs};

fn main() {
    let env = odra_casper_livenet_env::env();

    let owner = env.caller();

    println!("Block time: {}", env.block_time());

    // Funds can be transferred
    let another_account = env.get_account(1);
    let another_account_balance = env.balance_of(&another_account);
    env.transfer(another_account, U512::from(10_000_000_000u64))
        .unwrap();
    assert_eq!(
        env.balance_of(&another_account),
        another_account_balance + U512::from(10_000_000_000u64)
    );

    // Contract can be deployed
    env.set_gas(500_000_000_000u64);
    println!("Balance of user: {}", env.balance_of(&owner));

    deploy_erc20(&env);
    let (contract, erc20) = deploy_new(&env);

    // Contract can be loaded
    let (mut contract, erc20) = load(&env, contract.address(), erc20.address());

    // Errors can be handled
    env.set_gas(10_000_000_000u64);
    let r = contract.try_function_that_reverts();
    assert!(r.is_err());
    // TODO: we should be able to assert the error type here, but currently we can't because of the way errors are handled in Livenet environment.
    // The current error matching logic in Livenet env is based on error codes, which are not unique across contracts.
    // In a real project the codes are rather uniqe, but in `examples` we have a lot of contracts with small error codes, so the matching is not working as expected.
    // assert_eq!(r.unwrap_err(), SillyError.into());

    // There are three ways contract endpoints can be called in Livenet environment:
    // 1. If the endpoint is mutable and does not return anything, it can be called directly:
    assert_eq!(contract.get_stack_len(), 0);

    // 2. If the endpoint is mutable and returns something, it can be called through the proxy:
    contract.push_on_stack(1);
    let value = contract.pop_from_stack();
    assert_eq!(value, 1);

    // 3. If the endpoint is immutable, it can be called locally, querying only storage from livenet:
    assert_eq!(contract.owner(), owner);

    // By querying livenet storage
    // - we can also test the events
    assert_eq!(env.events_count(&contract), 1);

    let event: OwnershipTransferred = env.get_event(&contract, 0).unwrap();
    assert_eq!(event.new_owner, Some(owner));

    // - we can test immutable crosscalls without deploying (but crosscall contracts needs to be registered)
    assert_eq!(contract.immutable_cross_call(), 10_000.into());

    // wait some time to avoid node throttling
    std::thread::sleep(Duration::from_secs(5));

    // - mutable crosscalls will require a deploy
    let pre_call_balance = erc20.balance_of(&env.caller());
    contract.mutable_cross_call();
    let post_call_balance = erc20.balance_of(&env.caller());
    assert_eq!(post_call_balance, pre_call_balance + 1);

    // We can change the caller
    env.set_caller(env.get_account(1));

    // And query the balance
    println!("Balance of caller: {}", env.balance_of(&env.caller()));

    env.set_gas(500_000_000_000u64);

    // Contracts can be upgraded
    let mut counter =
        CounterV1::deploy_with_cfg(&env, NoArgs, InstallConfig::upgradable::<CounterV1>());

    counter.increment();
    assert_eq!(counter.get(), 1);

    let counter2 = CounterV2::try_upgrade(
        &env,
        counter.contract_address(),
        CounterV2UpgradeArgs { new_start: None }
    )
    .unwrap();

    assert_eq!(counter2.get(), U256::one());
    assert_eq!(counter2.get_old(), 1);
}

fn deploy_new(env: &HostEnv) -> (LivenetContractHostRef, Erc20HostRef) {
    let mut erc20_contract = deploy_erc20(env);
    let init_args = LivenetContractInitArgs {
        erc20_address: erc20_contract.address()
    };
    let livenet_contract = LivenetContract::deploy(env, init_args);
    erc20_contract.transfer(&livenet_contract.address(), &1000.into());
    (livenet_contract, erc20_contract)
}

fn load(
    env: &HostEnv,
    contract_address: Address,
    erc20_address: Address
) -> (LivenetContractHostRef, Erc20HostRef) {
    (
        LivenetContract::load(env, contract_address),
        Erc20::load(env, erc20_address)
    )
}

/// Deploys an ERC20 contract
pub fn deploy_erc20(env: &HostEnv) -> Erc20HostRef {
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 10u8;
    let initial_supply = Some(U256::from(10_000));

    let init_args = Erc20InitArgs {
        name,
        symbol,
        decimals,
        initial_supply
    };

    Erc20::deploy(env, init_args)
}
