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

#[cfg(test)]
mod test {
    use crate::features::upgrade::{
        CounterV1, CounterV2, CounterV2UpgradeArgs, IncrementEvent, IncrementEventV2
    };
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, InstallConfig, NoArgs};
    use odra::prelude::Addressable;

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

    /// A contract installed before the addressable-entity switch must be
    /// upgradable after it: the upgrade triggers the lazy migration of the
    /// legacy package and then adds (and disables) versions on the migrated
    /// package. State written pre-switch must be preserved. Runs only on the
    /// casper backend booted with `ODRA_CASPER_LEGACY_GENESIS=1`.
    #[test]
    fn upgrade_across_ae_switch() {
        let test_env = odra_test::env();
        let mut counter = CounterV1::deploy_with_cfg(
            &test_env,
            NoArgs,
            InstallConfig::new::<CounterV1>(true, true)
        );
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);

        if !test_env.enable_addressable_entity() {
            return;
        }

        let mut counter2 = CounterV2::try_upgrade(
            &test_env,
            counter.address(),
            CounterV2UpgradeArgs { new_start: None }
        )
        .unwrap();

        // Pre-switch state is preserved by the upgrade.
        assert_eq!(counter2.get(), U256::from(2));
        assert_eq!(counter2.get_old(), 2);

        // The new version works and emits readable events.
        counter2.increment();
        assert_eq!(counter2.get(), U256::from(3));
        assert!(counter2.env().emitted_event(
            &counter2,
            IncrementEventV2 {
                value: U256::from(3)
            }
        ));
    }
}
