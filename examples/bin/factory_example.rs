#![allow(missing_docs)]

use odra::{
    host::{Deployer, HostRefLoader, InstallConfig, NoArgs},
    prelude::*
};
use odra_examples::factory::counter::{
    BetterCounterFactory, BetterCounterUpgradeArgs, Counter, CounterFactory
};

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
    let args = vec![
        ("FromTwo".to_string(), 42u32),
        ("FromThree".to_string(), 42u32),
        ("FromHundred".to_string(), 1000u32),
    ]
    .into_iter()
    .map(|(contract_name, new_value)| (contract_name, BetterCounterUpgradeArgs { new_value }))
    .collect::<BTreeMap<_, _>>();
    new_factory.batch_upgrade_child_contract(args);
    assert_eq!(Counter::load(&env, from_two_address).value(), 42);
    assert_eq!(Counter::load(&env, from_three_address).value(), 42);
    assert_eq!(Counter::load(&env, from_hundred_address).value(), 1000);
}
