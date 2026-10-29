//! A factory that creates the contracts of a dapp.
//!
//! `DappCounterSpawner` keeps track of the counters of a dapp and creates them through
//! `DappCounterFactory`. The deployer of the factory lets only the spawner create counters
//! (`set_factory_creator`) and points the factory to the dapp's `OwnedDappRegistry`
//! (`set_factory_dapp_registry`), where the factory is a registrar: every counter the
//! spawner creates points to the registry and joins the dapp, and nobody else can add contracts
//! through the factory.
//!
//! The counters know nothing about the dapp: the factory points them to the registry. The spawner
//! points to it with `DappContractBase`.
use odra::{prelude::*, ContractRef};
use odra_modules::dapp::DappContractBase;

/// A counter created by the factory.
#[odra::module(factory=on)]
pub struct DappCounter {
    value: Var<u32>
}

#[odra::module(factory=on)]
impl DappCounter {
    /// Increments the counter.
    pub fn increment(&mut self) {
        self.value.add(1);
    }

    /// Returns the counter value.
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }
}

/// Creates counters through the factory.
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

    /// Creates a new counter. The factory points it to the dapp registry and adds it to the dapp.
    pub fn spawn(&mut self) -> Address {
        let factory = self.factory.get().unwrap_or_revert(self);
        let n = self.spawned.get_or_default();
        // The name keys the child in the factory, so each child gets its own.
        let mut factory = DappCounterFactoryContractRef::new(self.env(), factory);
        let (address, _) = factory.new_contract(format!("DappCounter{}", n));
        self.spawned.set(n + 1);
        address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostEnv, HostRef, NoArgs};
    use odra::prelude::ExecutionError;
    use odra_modules::dapp::errors::Error;
    use odra_modules::dapp::events::DappContractAdded;
    use odra_modules::dapp::{DappContractHostRef, OwnedDappRegistry, OwnedDappRegistryHostRef};

    fn setup() -> (
        HostEnv,
        OwnedDappRegistryHostRef,
        DappCounterFactoryHostRef,
        DappCounterSpawnerHostRef
    ) {
        let env = odra_test::env();
        let registry = OwnedDappRegistry::deploy(&env, NoArgs);
        let factory = DappCounterFactory::deploy(&env, NoArgs);
        let spawner = DappCounterSpawner::deploy(
            &env,
            DappCounterSpawnerInitArgs {
                registry: registry.address(),
                factory: factory.address()
            }
        );
        (env, registry, factory, spawner)
    }

    #[test]
    fn factory_adds_children_to_the_dapp() {
        let (env, mut registry, mut factory, mut spawner) = setup();
        // The factory points to the registry before the registry accepts it.
        factory.set_factory_creator(spawner.address());
        assert_eq!(
            registry.try_add_dapp_contract(&factory.address(), true),
            Err(ExecutionError::DappRegistryNotSet.into())
        );
        factory.set_factory_dapp_registry(registry.address());
        assert_eq!(factory.get_dapp_registry(), registry.address());
        registry.add_dapp_contract(&factory.address(), true);
        registry.add_dapp_contract(&spawner.address(), false);

        let first = spawner.spawn();
        let second = spawner.spawn();
        assert_ne!(first, second);
        assert_eq!(spawner.spawned(), 2);

        assert_eq!(
            registry.get_dapp_contracts(),
            vec![
                registry.address(),
                factory.address(),
                spawner.address(),
                first,
                second
            ]
        );
        assert!(!registry.is_dapp_registrar(&first));
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: first,
                is_registrar: false,
                registrar: factory.address()
            }
        ));

        let mut counter = DappCounterHostRef::new(first, env.clone());
        counter.increment();
        assert_eq!(counter.value(), 1);

        // The counters point to the registry without code of their own.
        let counter = DappContractHostRef::new(first, env.clone());
        assert_eq!(counter.get_dapp_registry(), registry.address());
    }

    #[test]
    fn factory_without_a_registry_adds_nothing() {
        let (env, mut registry, mut factory, mut spawner) = setup();
        factory.set_factory_creator(spawner.address());
        let counter = spawner.spawn();
        assert_eq!(registry.get_dapp_contracts(), vec![registry.address()]);

        // The counter does not point to a registry, so the registry does not accept it.
        let counter_ref = DappContractHostRef::new(counter, env.clone());
        assert_eq!(
            counter_ref.try_get_dapp_registry(),
            Err(ExecutionError::DappRegistryNotSet.into())
        );
        assert_eq!(
            registry.try_add_dapp_contract(&counter, false),
            Err(ExecutionError::DappRegistryNotSet.into())
        );

        // Upgrading the counter after the factory got a registry points it to the registry.
        factory.set_factory_dapp_registry(registry.address());
        factory.upgrade_child_contract("DappCounter0".to_string());
        assert_eq!(counter_ref.get_dapp_registry(), registry.address());
        registry.add_dapp_contract(&counter, false);

        // A counter pointing to another registry is rejected.
        let mut other = OwnedDappRegistry::deploy(&env, NoArgs);
        assert_eq!(
            other.try_add_dapp_contract(&counter, false),
            Err(Error::DappRegistryMismatch.into())
        );
    }

    #[test]
    fn a_child_keeps_its_registry() {
        let (env, mut registry, mut factory, mut spawner) = setup();
        factory.set_factory_creator(spawner.address());
        factory.set_factory_dapp_registry(registry.address());
        registry.add_dapp_contract(&factory.address(), true);
        let counter = spawner.spawn();

        // Pointing the factory to another registry moves only the children created afterwards: on
        // Casper an upgrade cannot replace the child's `__dapp` named key.
        let other = OwnedDappRegistry::deploy(&env, NoArgs);
        factory.set_factory_dapp_registry(other.address());
        factory.upgrade_child_contract("DappCounter0".to_string());
        let counter = DappContractHostRef::new(counter, env.clone());
        assert_eq!(counter.get_dapp_registry(), registry.address());
    }

    #[test]
    fn creator_restricts_new_contract() {
        let (env, _registry, mut factory, mut spawner) = setup();
        let (deployer, alice) = (env.get_account(0), env.get_account(1));

        // Without a creator anyone may create children.
        env.set_caller(alice);
        factory.new_contract("ByAlice".to_string());

        // Only the deployer holds the factory access.
        assert!(factory.try_set_factory_creator(alice).is_err());
        env.set_caller(deployer);
        factory.set_factory_creator(spawner.address());
        assert!(env.emitted_event(
            &factory,
            __dapp_counter_factory_dapp::FactoryCreatorChanged {
                previous_creator: None,
                new_creator: spawner.address()
            }
        ));

        // Now only the creator may.
        for caller in [deployer, alice] {
            env.set_caller(caller);
            assert_eq!(
                factory.try_new_contract("Direct".to_string()),
                Err(ExecutionError::CallerNotFactoryCreator.into())
            );
        }
        spawner.spawn();
        assert_eq!(spawner.spawned(), 1);
    }

    #[test]
    fn registry_needs_a_creator() {
        let (env, registry, mut factory, spawner) = setup();

        // Without a creator anyone could add contracts to the dapp through the factory.
        assert_eq!(
            factory.try_set_factory_dapp_registry(registry.address()),
            Err(ExecutionError::FactoryCreatorNotSet.into())
        );

        factory.set_factory_creator(spawner.address());
        assert_eq!(
            factory.try_set_factory_dapp_registry(env.get_account(1)),
            Err(ExecutionError::DappRegistryNotAContract.into())
        );
        env.set_caller(env.get_account(1));
        assert!(factory
            .try_set_factory_dapp_registry(registry.address())
            .is_err());
        env.set_caller(env.get_account(0));
        factory.set_factory_dapp_registry(registry.address());
        assert!(env.emitted_event(
            &factory,
            __dapp_counter_factory_dapp::DappRegistryChanged {
                previous_registry: None,
                new_registry: registry.address()
            }
        ));
    }
}
