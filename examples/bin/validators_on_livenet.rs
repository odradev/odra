//! Deploys an example Validators contract and tests its functionality.
use core::time::Duration;
use odra::casper_types::{PublicKey, SecretKey, U512};
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

    // The validator's bid is read from the chain; an unknown validator has none.
    let minimum_delegation_amount = validators_contract.get_minimum_delegation_amount();
    println!("Minimum delegation amount: {minimum_delegation_amount}");
    let unknown = PublicKey::from(&SecretKey::ed25519_from_bytes([7u8; 32]).unwrap());
    let unknown_validator_contract =
        ValidatorsContract::deploy(&env, ValidatorsContractInitArgs { validator: unknown });
    assert_eq!(
        unknown_validator_contract.try_get_minimum_delegation_amount(),
        Err(ExecutionError::UnwrapError.into())
    );

    // Stake some amount
    let staking_amount = U512::from(1_000_000_000_000u64);
    validators_contract.with_tokens(staking_amount).stake();

    // Compare delegated amount from contract and from host env
    let delegated_amount_contract = validators_contract.currently_delegated_amount();
    let delegated_amount_host = env.delegated_amount(validators_contract.address(), validator);
    assert_eq!(delegated_amount_contract, delegated_amount_host);

    // Check Host's validator's functionality
    println!("Auction delay: {} ms", env.auction_delay());
    println!("Unbonding delay: {} ms", env.unbonding_delay());

    env.advance_with_auctions(Duration::from_secs(1));
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
