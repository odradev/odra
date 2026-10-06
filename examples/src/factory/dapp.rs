//! A factory that spawns contracts belonging to a dapp.
//!
//! `DappCounterSpawner` is registered in an `OwnedDappRegistry` as a factory. Each `spawn` deploys a
//! new `DappCounter` through `DappCounterFactory`, pointed at the same registry, and registers it there.
use odra::{prelude::*, ContractRef};
use odra_modules::dapp::{DappContractBase, DappRegistryContractRef};

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

/// Deploys counters through the factory and registers them in the dapp.
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

    /// Deploys a new counter and registers it in the dapp.
    ///
    /// The spawner must be registered in the dapp as a factory.
    pub fn spawn(&mut self) -> Address {
        let registry = self.dapp.get_dapp_registry();
        let factory = self.factory.get().unwrap_or_revert(self);
        let n = self.spawned.get_or_default();
        // The name keys the child in the factory, so each child gets its own.
        let mut factory = DappCounterFactoryContractRef::new(self.env(), factory);
        let (address, _) = factory.new_contract(format!("DappCounter{}", n), registry);
        self.spawned.set(n + 1);

        DappRegistryContractRef::new(self.env(), registry).add_dapp_contract(&address, false);
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
    fn factory_registers_spawned_contracts() {
        // Factories work on CasperVM only; `cargo odra test -b casper` runs this test.
        if std::env::var("ODRA_BACKEND").as_deref() != Ok("casper") {
            return;
        }
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

        // The spawner is not a factory of the dapp yet.
        assert_eq!(spawner.try_spawn(), Err(Error::CallerNotDappFactory.into()));

        registry.add_dapp_contract(&spawner.address(), true);
        let first = spawner.spawn();
        let second = spawner.spawn();
        assert_ne!(first, second);
        assert_eq!(spawner.spawned(), 2);

        assert_eq!(
            registry.get_dapp_contracts(),
            vec![registry.address(), spawner.address(), first, second]
        );
        assert!(!registry.is_dapp_factory(&first));
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: first,
                is_factory: false,
                registrar: spawner.address()
            }
        ));

        let mut counter = DappCounterHostRef::new(first, env.clone());
        assert_eq!(counter.get_dapp_registry(), registry.address());
        counter.increment();
        assert_eq!(counter.value(), 1);
    }
}
