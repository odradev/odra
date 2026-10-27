//! A factory that creates the contracts of a dapp.
//!
//! `DappCounterSpawner` keeps track of the counters of a dapp and creates them through
//! `DappCounterFactory`. The deployer of the factory lets only the spawner create counters
//! (`set_factory_creator`) and points the factory to the dapp's `OwnedDappRegistry`
//! (`set_factory_dapp_registry`), where the factory is a registrar: every counter the
//! spawner creates joins the dapp, and nobody else can add contracts through the factory.
//!
//! The counters know nothing about the dapp. The spawner confirms its membership from its side with
//! `DappMemberBase`, which users can check against the registry's list.
use odra::{prelude::*, ContractRef};
use odra_modules::dapp::DappMemberBase;

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
    dapp: SubModule<DappMemberBase>,
    factory: Var<Address>,
    spawned: Var<u32>
}

#[odra::module]
impl DappCounterSpawner {
    /// Points the spawner to the counter factory; the spawner recognizes the dapp registry.
    pub fn init(&mut self, registry: Address, factory: Address) {
        self.dapp.init();
        self.dapp.recognize_dapp_registry(&registry);
        self.factory.set(factory);
    }

    /// Returns the number of spawned counters.
    pub fn spawned(&self) -> u32 {
        self.spawned.get_or_default()
    }

    delegate! {
        to self.dapp {
            fn recognizes_dapp_registry(&self, registry: &Address) -> bool;
        }
    }

    /// Creates a new counter. The factory adds it to the dapp.
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
    use odra_modules::dapp::events::DappContractAdded;
    use odra_modules::dapp::{OwnedDappRegistry, OwnedDappRegistryHostRef};

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
        factory.set_factory_creator(spawner.address());
        registry.add_dapp_contract(&factory.address(), true);
        factory.set_factory_dapp_registry(registry.address());
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

        // The spawner recognizes the dapp; the counters do not have to.
        assert!(spawner.recognizes_dapp_registry(&registry.address()));
    }

    #[test]
    fn factory_without_a_registry_adds_nothing() {
        let (_env, registry, mut factory, mut spawner) = setup();
        factory.set_factory_creator(spawner.address());
        spawner.spawn();
        assert_eq!(registry.get_dapp_contracts(), vec![registry.address()]);
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
        let (env, mut registry, mut factory, spawner) = setup();
        registry.add_dapp_contract(&factory.address(), true);

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
