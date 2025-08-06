//! This example demonstrates how to deploy and upgrade a contract on the Livenet environment.

use odra::casper_types::U256;
use odra::host::{Deployer, HostRef, InstallConfig, NoArgs};
use odra_examples::features::upgrade::{CounterV1, CounterV2, CounterV2UpgradeArgs};

fn main() {
    let env = odra_casper_livenet_env::env();

    env.set_gas(500_000_000_000u64);

    // Contracts can be upgraded
    let mut counter =
        CounterV1::deploy_with_cfg(&env, NoArgs, InstallConfig::upgradable::<CounterV1>());

    env.set_gas(50_000_000_000u64);
    counter.increment();
    assert_eq!(counter.get(), 1);

    env.set_gas(500_000_000_000u64);
    let mut counter2 = CounterV2::try_upgrade(
        &env,
        counter.contract_address(),
        CounterV2UpgradeArgs {
            new_start: None,
        }
    )
    .unwrap();

    env.set_gas(50_000_000_000u64);
    counter2.increment();
    assert_eq!(counter2.get(), U256::from(2));
    assert_eq!(counter2.get_old(), 1);
}
