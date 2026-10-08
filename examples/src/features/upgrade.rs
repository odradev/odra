//! This example shows how to upgrade a contract.

use odra::casper_types::U256;
use odra::prelude::*;

/// A Contract that counts, version 1
#[odra::module(events = [IncrementEvent])]
pub struct CounterV1 {
    counter: Var<u32>
}

#[odra::event]
pub struct IncrementEvent {
    pub value: u32
}

#[odra::module]
impl CounterV1 {
    pub fn init(&mut self) {
        self.counter.set(0);
    }

    pub fn increment(&mut self) {
        let counter = self.counter.get_or_default() + 1;
        self.counter.set(counter);
        self.env().emit_event(IncrementEvent { value: counter });
    }

    pub fn get(&self) -> u32 {
        self.counter.get_or_default()
    }

    pub fn reset(&mut self) {
        self.counter.set(0);
    }
}

#[odra::event]
pub struct IncrementEventV2 {
    pub value: U256
}

/// A Contract that counts, version 2
#[odra::module(events = [IncrementEventV2])]
pub struct CounterV2 {
    counter: Var<u32>,
    new_counter: Var<U256>
}
#[odra::module]
impl CounterV2 {
    pub fn init(&mut self, start_from: Option<U256>) {
        if let Some(start) = start_from {
            self.new_counter.set(start);
        } else {
            self.new_counter.set(U256::from(0));
        }
    }

    pub fn upgrade(&mut self, new_start: Option<U256>) {
        if let Some(start) = new_start {
            self.new_counter.set(start);
        } else {
            // If no new value is provided, we keep the current value
            self.new_counter.set(self.counter.get_or_default().into());
        }
    }

    pub fn increment(&mut self) {
        let counter = self.new_counter.get_or_default() + U256::one();
        self.new_counter.set(counter);
        self.env().emit_event(IncrementEventV2 { value: counter });
    }

    pub fn get(&self) -> U256 {
        self.new_counter.get_or_default()
    }

    pub fn get_old(&self) -> u32 {
        self.counter.get_or_default()
    }

    /// We intentionally don't implement this method.
    pub fn reset(&mut self) {}

    pub fn set(&mut self, value: U256) {
        self.new_counter.set(value);
    }
}

/// Errors of [CounterV3].
#[odra::odra_error]
pub enum UpgradeError {
    /// The upgrade was asked to fail.
    Refused = 1
}

/// A Contract that counts in tens, version 3. Its upgrade fails on request.
#[odra::module(errors = UpgradeError)]
pub struct CounterV3 {
    counter: Var<u32>
}

#[odra::module]
impl CounterV3 {
    pub fn upgrade(&mut self, fail: bool) {
        if fail {
            self.env().revert(UpgradeError::Refused)
        }
    }

    pub fn increment(&mut self) {
        self.counter.set(self.counter.get_or_default() + 10);
    }

    pub fn get(&self) -> u32 {
        self.counter.get_or_default()
    }
}

#[cfg(test)]
mod test {
    use crate::features::upgrade::{
        CounterV1, CounterV2, CounterV2UpgradeArgs, CounterV3, CounterV3UpgradeArgs,
        IncrementEvent, IncrementEventV2, UpgradeError
    };
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, InstallConfig, NoArgs, UpgradeConfig};
    use odra::prelude::*;

    #[test]
    fn it_works() {
        let test_env = odra_test::env();
        let mut counter = CounterV1::deploy_with_cfg(
            &test_env,
            NoArgs,
            InstallConfig::new::<CounterV1>(true, true)
        );

        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);
        assert_eq!(counter.env().events_count(&counter), 1);
        assert!(counter
            .env()
            .emitted_event(&counter, IncrementEvent { value: 1 }));

        counter.reset();
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        let mut counter2 = CounterV2::try_upgrade(
            &test_env,
            counter.address(),
            CounterV2UpgradeArgs { new_start: None }
        )
        .unwrap();

        assert_eq!(counter2.get(), U256::one());

        counter2.increment();
        assert_eq!(counter2.get(), U256::from(2));
        assert_eq!(counter.env().events_count(&counter), 3);
        assert!(counter.env().emitted_event(
            &counter,
            IncrementEventV2 {
                value: U256::from(2)
            }
        ));

        counter2.set(U256::from(100));
        assert_eq!(counter2.get(), U256::from(100));

        assert_eq!(counter2.get_old(), 1);

        let _counter3 = CounterV2::try_upgrade(
            &test_env,
            counter.address(),
            CounterV2UpgradeArgs { new_start: None }
        )
        .unwrap();
    }
    /// A contract installed under one package-hash key (e.g. `Struct_package_hash` by Odra 2.9)
    /// can be upgraded with a config that names the key differently (what a 2.10 `name = ".."`
    /// produces): the upgrade finds the package by address and is authorized by the account's
    /// access URef, not by the key name. Afterwards the package hash sits under both keys.
    #[test]
    fn upgrade_survives_a_package_key_rename() {
        let test_env = odra_test::env();
        let mut counter = CounterV1::deploy_with_cfg(
            &test_env,
            NoArgs,
            InstallConfig {
                package_named_key: String::from("OldName"),
                is_upgradable: true,
                allow_key_override: true
            }
        );
        counter.increment();
        assert_eq!(counter.get(), 1);

        let counter2 = CounterV2::try_upgrade_with_cfg(
            &test_env,
            counter.address(),
            CounterV2UpgradeArgs { new_start: None },
            UpgradeConfig {
                package_named_key: String::from("NewName"),
                force_create_upgrade_group: false,
                allow_key_override: true
            }
        )
        .unwrap();

        // Same package, state kept.
        assert_eq!(counter2.address(), counter.address());
        assert_eq!(counter2.get(), U256::one());
    }

    #[test]
    fn reverted_upgrade_keeps_the_old_code() {
        let test_env = odra_test::env();
        let mut counter =
            CounterV1::deploy_with_cfg(&test_env, NoArgs, InstallConfig::upgradable::<CounterV1>());
        counter.increment();

        let result = CounterV3::try_upgrade(
            &test_env,
            counter.address(),
            CounterV3UpgradeArgs { fail: true }
        );
        assert_eq!(result.err(), Some(UpgradeError::Refused.into()));

        // Still version 1: it counts in ones.
        counter.increment();
        assert_eq!(counter.get(), 2);

        let mut counter = CounterV3::try_upgrade(
            &test_env,
            counter.address(),
            CounterV3UpgradeArgs { fail: false }
        )
        .unwrap();
        counter.increment();
        assert_eq!(counter.get(), 12);
    }

    #[test]
    fn restoring_a_snapshot_brings_back_the_old_code() {
        let test_env = odra_test::env();
        let mut counter =
            CounterV1::deploy_with_cfg(&test_env, NoArgs, InstallConfig::upgradable::<CounterV1>());
        counter.increment();
        test_env.take_snapshot();

        counter.increment();
        let mut upgraded = CounterV3::try_upgrade(
            &test_env,
            counter.address(),
            CounterV3UpgradeArgs { fail: false }
        )
        .unwrap();
        upgraded.increment();
        assert_eq!(upgraded.get(), 12);
        let mut deployed = CounterV1::deploy(&test_env, NoArgs);
        deployed.increment();
        deployed.increment();

        test_env.restore_snapshot();

        // Version 1 again, and its events are still tracked.
        counter.increment();
        assert!(counter
            .last_call()
            .emitted_event(IncrementEvent { value: 2 }));
        assert_eq!(counter.get(), 2);

        // A contract deployed after the restore starts clean.
        let mut fresh = CounterV3::deploy(&test_env, NoArgs);
        fresh.increment();
        assert_eq!(fresh.get(), 10);
    }
}
