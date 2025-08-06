//! This example shows how to handle signature verification in a contract.
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

    pub fn upgrade(&mut self) {}

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
    pub fn init(&mut self, _lol: String) {}

    pub fn upgrade(&mut self, _miau: String) {
        self.new_counter.set(U256::from(0));
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
    pub fn reset(&mut self) {}

    pub fn set(&mut self, value: U256) {
        self.new_counter.set(value);
    }
}

#[odra::module(events = [IncrementEventV2])]
pub struct CounterV3 {
    #[allow(dead_code)]
    counter: Var<u32>,
    new_counter: Var<U256>
}

#[odra::module]
impl CounterV3 {
    pub fn increment(&mut self) {
        let counter = self.new_counter.get_or_default() + U256::one();
        self.new_counter.set(counter);
        self.env().emit_event(IncrementEventV2 { value: counter });
    }

    pub fn get(&self) -> U256 {
        self.new_counter.get_or_default()
    }

    pub fn set(&mut self, value: U256) {
        self.new_counter.set(value);
    }
}

#[cfg(test)]
mod test {
    use crate::alloc::string::ToString;
    use crate::features::upgrade::{
        CounterV1, CounterV2, CounterV2UpgradeArgs, CounterV3, IncrementEvent, IncrementEventV2
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
            CounterV2UpgradeArgs {
                _miau: "miau".to_string()
            }
        )
        .unwrap();

        assert_eq!(counter2.get(), U256::zero());

        counter2.increment();
        assert_eq!(counter2.get(), U256::from(1));
        assert_eq!(counter.env().events_count(&counter), 3);
        assert!(counter.env().emitted_event(
            &counter,
            IncrementEventV2 {
                value: U256::from(1)
            }
        ));

        counter2.set(U256::from(100));
        assert_eq!(counter2.get(), U256::from(100));

        assert_eq!(counter2.get_old(), 1);

        let counter3 = CounterV3::try_upgrade(&test_env, counter2.address(), NoArgs).unwrap();

        assert_eq!(counter3.get(), U256::from(100));
    }
}
