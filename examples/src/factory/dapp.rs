//! A factory that creates the contracts of a dapp.
//!
//! `DappCounterSpawner` keeps track of the counters of a dapp and creates them through
//! `DappCounterFactory`. The spawner belongs to the dapp's `OwnedDappRegistry` as a registrar: it
//! points every counter it creates to the registry (`register_address`) and adds it there (`register`).
use odra::{prelude::*, ContractRef};
use odra_modules::dapp::DappContractBase;

/// A counter that belongs to a dapp.
#[odra::module(factory=on)]
pub struct DappCounter {
    dapp: SubModule<DappContractBase>,
    value: Var<u32>
}

#[odra::module(factory=on)]
impl DappCounter {
    /// Points the counter to the dapp registry.
    pub fn init(&mut self, registry: Address) {
        self.dapp.set_dapp_registry(&registry);
    }

    /// Increments the counter.
    pub fn increment(&mut self) {
        self.value.add(1);
    }

    /// Returns the counter value.
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }

    delegate! {
        to self.dapp {
            fn get_dapp_registry(&self) -> Address;
        }
    }
}

/// Creates counters through the factory and adds them to the dapp.
#[odra::module]
pub struct DappCounterSpawner {
    dapp: SubModule<DappContractBase>,
    factory: Var<Address>,
    spawned: Var<u32>
}

#[odra::module]
impl DappCounterSpawner {
    /// Points the spawner to the dapp registry and to the counter factory.
    pub fn init(&mut self, registry: Address, factory: Address) {
        self.dapp.set_dapp_registry(&registry);
        self.factory.set(factory);
    }

    /// Returns the number of spawned counters.
    pub fn spawned(&self) -> u32 {
        self.spawned.get_or_default()
    }

    delegate! {
        to self.dapp {
            fn get_dapp_registry(&self) -> Address;
        }
    }

    /// Creates a new counter pointing to the dapp registry and adds it to the dapp.
    ///
    /// The spawner must be registered in the dapp as a registrar.
    pub fn spawn(&mut self) -> Address {
        let factory = self.factory.get().unwrap_or_revert(self);
        let n = self.spawned.get_or_default();
        // The name keys the child in the factory, so each child gets its own.
        let mut factory = DappCounterFactoryContractRef::new(self.env(), factory);
        let (address, _) =
            factory.new_contract(format!("DappCounter{}", n), self.dapp.register_address());
        self.spawned.set(n + 1);
        self.dapp.register(address);
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef, NoArgs};
    use odra_modules::dapp::errors::Error;
    use odra_modules::dapp::events::DappContractAdded;
    use odra_modules::dapp::OwnedDappRegistry;

    #[test]
    fn spawner_adds_counters_to_the_dapp() {
        let env = odra_test::env();
        let mut registry = OwnedDappRegistry::deploy(&env, NoArgs);
        let factory = DappCounterFactory::deploy(&env, NoArgs);
        let mut spawner = DappCounterSpawner::deploy(
            &env,
            DappCounterSpawnerInitArgs {
                registry: registry.address(),
                factory: factory.address()
            }
        );

        // The spawner is not a registrar of the dapp yet.
        assert_eq!(
            spawner.try_spawn(),
            Err(Error::CallerNotDappRegistrar.into())
        );

        registry.add_dapp_contract(&spawner.address(), true);
        let first = spawner.spawn();
        let second = spawner.spawn();
        assert_ne!(first, second);
        assert_eq!(spawner.spawned(), 2);

        assert_eq!(
            registry.get_dapp_contracts(),
            vec![registry.address(), spawner.address(), first, second]
        );
        assert!(!registry.is_dapp_registrar(&first));
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: first,
                is_registrar: false,
                registrar: spawner.address()
            }
        ));

        let mut counter = DappCounterHostRef::new(first, env.clone());
        assert_eq!(counter.get_dapp_registry(), registry.address());
        counter.increment();
        assert_eq!(counter.value(), 1);
    }
}
