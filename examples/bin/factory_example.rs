#![allow(missing_docs)]

use odra::{
    casper_types::{
        bytesrepr::{Bytes, ToBytes},
        runtime_args
    },
    host::{Deployer, HostRefLoader, InstallConfig, NoArgs},
    prelude::*
};
use odra_examples::factory::counter::{BetterCounterFactory, Counter, CounterFactory};

fn main() {
    let env = odra_casper_livenet_env::env();

    env.set_gas(480_000_000_000u64);
    let mut factory = CounterFactory::deploy_with_cfg(
        &env,
        NoArgs,
        InstallConfig {
            package_named_key: "TestFactory".to_string(),
            is_upgradable: true,
            allow_key_override: true
        }
    );

    env.set_gas(270_000_000_000u64);
    let (from_ten_address, _) = factory.new_contract(String::from("FromTen"), 10);
    let (from_two_address, _) = factory.new_contract(String::from("FromTwo"), 2);
    let (from_three_address, _) = factory.new_contract(String::from("FromThree"), 3);
    let (from_hundred_address, _) = factory.new_contract(String::from("FromHundred"), 100);

    env.set_gas(500_000_000_000u64);
    let result = BetterCounterFactory::try_upgrade(&env, factory.address(), NoArgs);
    assert!(result.is_ok());

    let mut new_factory = result.unwrap();
    new_factory.upgrade_child_contract(String::from("FromTen"), 122);
    assert_eq!(Counter::load(&env, from_ten_address).value(), 122);

    env.set_gas(900_000_000_000u64);
    new_factory.batch_upgrade_child_contract(
        Bytes::from(
            runtime_args! {
                "new_value" => 42u32
            }
            .to_bytes()
            .expect("Failed to serialize runtime args for default_args")
        ),
        Bytes::from(
            vec![
                "FromTwo".to_string(),
                "FromThree".to_string(),
                "FromHundred".to_string(),
            ]
            .to_bytes()
            .expect("Failed to serialize runtime args for names_to_upgrade")
        ),
        Bytes::from(
            BTreeMap::from([(
                "FromHundred".to_string(),
                runtime_args! {
                    "new_value" => 1000u32
                }
            )])
            .to_bytes()
            .expect("Failed to serialize runtime args for specific_args")
        )
    );
    assert_eq!(Counter::load(&env, from_two_address).value(), 42);
    assert_eq!(Counter::load(&env, from_three_address).value(), 42);
    assert_eq!(Counter::load(&env, from_hundred_address).value(), 1000);
}
