#![allow(missing_docs)]

use odra::{
    casper_types::runtime_args,
    host::{Deployer, FactoryUpgradeArgs, HostRef, HostRefLoader, InstallConfig, NoArgs},
    prelude::*
};
use odra_examples::factory::counter::{
    BetterCounterFactory, Counter, CounterFactory, CounterFactoryHostRef, CounterHostRef
};

fn main() {
    let env = odra_casper_livenet_env::env();

    env.set_gas(550_000_000_000u64);
    let mut factory_ref = CounterFactory::deploy_with_cfg(
        &env,
        NoArgs,
        InstallConfig {
            package_named_key: "TestFactory".to_string(),
            is_upgradable: true,
            allow_key_override: true
        }
    );

    env.set_gas(300_000_000_000u64);
    let (from_ten_address, _access_uref) = factory_ref.factory(String::from("FromTen"), 10);
    let (from_two_address, _access_uref) = factory_ref.factory(String::from("FromTwo"), 2);
    let (from_three_address, _access_uref) = factory_ref.factory(String::from("FromThree"), 3);
    let (from_hundred_address, _access_uref) =
        factory_ref.factory(String::from("FromHundred"), 100);

    let args = FactoryUpgradeArgs {
        default_args: runtime_args! {
            "new_value" => 42u32
        },
        names_to_upgrade: vec![
            "FromTen".to_string(),
            "FromTwo".to_string(),
            "FromThree".to_string(),
            "FromHundred".to_string(),
        ],
        specific_args: [
            (
                "FromTen".to_string(),
                runtime_args! {
                    "new_value" => 122u32
                }
            ),
            (
                "FromHundred".to_string(),
                runtime_args! {
                    "new_value" => 1000u32
                }
            )
        ]
        .into()
    };
    // Upgrade the factory contract
    env.set_gas(2_200_000_000_000u64);
    let result = BetterCounterFactory::try_upgrade(&env, factory_ref.address(), args);
    assert!(result.is_ok());

    assert_eq!(Counter::load(&env, from_ten_address).value(), 122);
    assert_eq!(Counter::load(&env, from_two_address).value(), 42);
    assert_eq!(Counter::load(&env, from_three_address).value(), 42);
    assert_eq!(Counter::load(&env, from_hundred_address).value(), 1000);
}
