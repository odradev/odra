//! This example demonstrates how to deploy and interact with a contract on the Livenet environment.
use odra::casper_types::{U256, U512};
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader};
use odra::{Address, ExecutionError};
use odra_examples::features::livenet::{LivenetContractHostRef, LivenetContractInitArgs};
use odra_modules::access::events::OwnershipTransferred;
use odra_modules::erc20::{Erc20HostRef, Erc20InitArgs};

fn main() {
    let env = odra_casper_livenet_env::env();

    let owner = env.caller();

    println!("Block time: {}", env.block_time());

    // Funds can be transferred
    // let another_account = env.get_account(1);
    // let another_account_balance = env.balance_of(&another_account);
    // env.transfer(another_account.clone(), U512::from(10_000_000_000u64))
    //     .unwrap();
    // assert_eq!(
    //     env.balance_of(&another_account),
    //     another_account_balance + U512::from(10_000_000_000u64)
    // );

    // Contract can be deployed
    env.set_gas(30_000_000_000u64);
    let (contract, erc20) = deploy_new(&env);

    // Contract can be loaded
    let (mut contract, erc20) = load(&env, *contract.address(), *erc20.address());

    // Errors can be handled
    // env.set_gas(1u64);
    // TODO: Fix setting gas for contract calls
    // let result = contract.try_push_on_stack(1).unwrap_err();
    // assert_eq!(result, ExecutionError::OutOfGas.into());
    contract.push_on_stack(1);
    let r = contract.try_function_that_reverts();

    // Set gas will be used for all subsequent calls
    env.set_gas(1_000_000_000u64);

    // There are three ways contract endpoints can be called in Livenet environment:
    // 1. If the endpoint is mutable and does not return anything, it can be called directly:
    assert_eq!(contract.get_stack_len(), 1);

    // 2. If the endpoint is mutable and returns something, it can be called through the proxy:
    contract.push_on_stack(1);
    let value = contract.pop_from_stack();
    assert_eq!(value, 1);

    // 3. If the endpoint is immutable, it can be called locally, querying only storage from livenet:
    assert_eq!(contract.owner(), owner);

    // By querying livenet storage
    // - we can also test the events
    assert_eq!(env.events_count(contract.address()), 1);

    let event: OwnershipTransferred = env.get_event(contract.address(), 0).unwrap();
    assert_eq!(event.new_owner, Some(owner));

    // - we can test immutable crosscalls without deploying (but crosscall contracts needs to be registered)
    assert_eq!(contract.immutable_cross_call(), 10_000.into());

    // - mutable crosscalls will require a deploy
    let pre_call_balance = erc20.balance_of(&env.caller());
    contract.mutable_cross_call();
    let post_call_balance = erc20.balance_of(&env.caller());
    assert_eq!(post_call_balance, pre_call_balance + 1);

    // We can change the caller
    env.set_caller(env.get_account(1));

    // And query the balance
    println!("Balance of caller: {}", env.balance_of(&env.caller()));
}

fn deploy_new(env: &HostEnv) -> (LivenetContractHostRef, Erc20HostRef) {
    let mut erc20_contract = deploy_erc20(env);
    env.set_gas(100_000_000_000u64);
    let init_args = LivenetContractInitArgs {
        erc20_address: *erc20_contract.address()
    };
    let livenet_contract = LivenetContractHostRef::deploy(env, init_args);
    erc20_contract.transfer(livenet_contract.address(), &1000.into());
    (livenet_contract, erc20_contract)
}

fn load(
    env: &HostEnv,
    contract_address: Address,
    erc20_address: Address
) -> (LivenetContractHostRef, Erc20HostRef) {
    (
        LivenetContractHostRef::load(env, contract_address),
        Erc20HostRef::load(env, erc20_address)
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

    env.set_gas(100_000_000_000u64);
    Erc20HostRef::deploy(env, init_args)
}
