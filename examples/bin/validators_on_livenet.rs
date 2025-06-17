//! Deploys an example Validators contract and tests its functionality.
use odra::casper_types::{PublicKey, U512};
use odra::host::{Deployer, HostEnv, HostRef};
use odra::prelude::*;
use odra_examples::features::validators::{
    ValidatorsContract, ValidatorsContractHostRef, ValidatorsContractInitArgs
};

fn main() {
    let env = odra_casper_livenet_env::env();

    // Deploy new contract.
    env.set_gas(300_000_000_000u64);
    let (validators_contract, validator) = deploy_validators(&env);
    println!(
        "Validators address: {}",
        validators_contract.address().to_string()
    );

    // Stake some amount
    let staking_amount = U512::from(1_000_000_000_000u64);
    validators_contract.with_tokens(staking_amount).stake();

    // Compare delegated amount from contract and from host env
    let delegated_amount_contract = validators_contract.currently_delegated_amount();
    let delegated_amount_host = env.delegated_amount(validators_contract.address(), validator);
    assert_eq!(delegated_amount_contract, delegated_amount_host);

    // Check Host's validator's functionality
    println!("Auction delay: {:?}", env.auction_delay());
    println!("Unbonding delay: {:?}", env.unbonding_delay());

    env.advance_with_auctions(1000);
}

/// Deploys an ERC20 contract.
pub fn deploy_validators(env: &HostEnv) -> (ValidatorsContractHostRef, PublicKey) {
    let validator = env.get_validator(0);
    (
        ValidatorsContract::deploy(
            env,
            ValidatorsContractInitArgs {
                validator: validator.clone()
            }
        ),
        validator
    )
}
