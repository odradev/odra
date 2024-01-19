use odra::{Address, HostEnv};
use odra_examples::features::module_nesting::{
    NestedOdraTypesContractDeployer, NestedOdraTypesContractHostRef, OperationResult, Status
};
use std::process::exit;
use std::str::FromStr;

fn main() {
    let env = odra_casper_livenet_env::livenet_env();

    let owner = env.caller();
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // Uncomment to deploy new contract.
    // let mut token = deploy_new(&env);
    // println!("Token address: {}", token.address().to_string());

    // Load existing contract.
    let mut token = load(&env);

    println!("Current generation: {:?}", token.current_generation());
    println!("Latest result: {:?}", token.latest_result());

    exit(0);

    println!("Saving operation result");
    env.set_gas(3_000_000_000u64);
    token.save_operation_result(OperationResult {
        id: 0,
        status: Status::Success,
        description: "Zero is a success".to_string()
    });

    println!("Current generation: {:?}", token.current_generation());
    println!("Latest result: {:?}", token.latest_result());
}

fn _deploy_new(env: &HostEnv) -> NestedOdraTypesContractHostRef {
    env.set_gas(100_000_000_000u64);
    NestedOdraTypesContractDeployer::init(env)
}

fn load(env: &HostEnv) -> NestedOdraTypesContractHostRef {
    let address = "hash-da858ca065eb039b10085467673ee896f853fdc5e629b38ae3f7746e3bab4dca";
    let address = Address::from_str(address).unwrap();
    NestedOdraTypesContractDeployer::load(env, address)
}
