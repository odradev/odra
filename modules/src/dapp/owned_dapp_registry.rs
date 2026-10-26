//! A dapp registry owned by a single account.
use super::{DappMetadata, DappRegistryBase};
use crate::access::errors::Error as AccessError;
use crate::access::Ownable;
use odra::prelude::*;

/// A ready-to-deploy dapp registry built from [`Ownable`] and [`DappRegistryBase`].
///
/// The deployer becomes the owner. The owner sets the metadata and adds or removes contracts. A contract
/// the owner registered as a factory may add contracts too, but not further factories.
#[odra::module]
pub struct OwnedDappRegistry {
    ownable: SubModule<Ownable>,
    dapp: SubModule<DappRegistryBase>
}

#[odra::module]
impl OwnedDappRegistry {
    /// Initializes the registry, making the caller its owner.
    pub fn init(&mut self) {
        let owner = self.env().caller();
        self.ownable.init(owner);
        self.dapp.init();
    }

    /// Sets the dapp metadata. Only the owner may call it.
    pub fn set_dapp_metadata(&mut self, metadata: DappMetadata) {
        self.assert_owner();
        self.dapp.set_dapp_metadata(metadata);
    }

    /// Adds a contract to the dapp.
    ///
    /// The owner may add any contract. A registered factory may add contracts that are not factories.
    /// Anyone else is rejected with `CallerNotTheOwner`, and an unregistered contract with
    /// `CallerNotDappFactory`.
    pub fn add_dapp_contract(&mut self, dapp_contract: &Address, is_factory: bool) {
        let caller = self.env().caller();
        if self.ownable.get_optional_owner() != Some(caller) {
            if is_factory || !caller.is_contract() {
                self.env().revert(AccessError::CallerNotTheOwner);
            }
            self.dapp.assert_dapp_factory(&caller);
        }
        self.dapp.add_dapp_contract(dapp_contract, is_factory);
    }

    /// Removes a contract from the dapp. Only the owner may call it.
    pub fn remove_dapp_contract(&mut self, dapp_contract: &Address) {
        self.assert_owner();
        self.dapp.remove_dapp_contract(dapp_contract);
    }

    delegate! {
        to self.dapp {
            /// Returns the dapp metadata.
            fn get_dapp_metadata(&self) -> DappMetadata;
            /// Returns the contracts of the dapp, the registry first.
            fn get_dapp_contracts(&self) -> Vec<Address>;
            /// Returns true if the contract belongs to the dapp.
            fn is_dapp_contract(&self, dapp_contract: &Address) -> bool;
            /// Returns true if the contract is a registered factory.
            fn is_dapp_factory(&self, dapp_contract: &Address) -> bool;
        }
        to self.ownable {
            /// Returns the owner.
            fn get_owner(&self) -> Address;
            /// Transfers ownership to `new_owner`. Only the owner may call it.
            fn transfer_ownership(&mut self, new_owner: &Address);
            /// Leaves the registry without an owner. Only the owner may call it.
            fn renounce_ownership(&mut self);
        }
    }
}

impl OwnedDappRegistry {
    fn assert_owner(&self) {
        self.ownable.assert_owner(&self.env().caller());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dapp::errors::Error;
    use crate::dapp::events::{
        DappContractAdded, DappContractRemoved, DappMetadataChanged, DappRegistryForgotten,
        DappRegistryRecognized
    };
    use crate::dapp::utils::{MockDappContract, MockDappContractHostRef};
    use odra::host::{Deployer, HostEnv, NoArgs};

    fn setup() -> (HostEnv, OwnedDappRegistryHostRef) {
        let env = odra_test::env();
        let registry = OwnedDappRegistry::deploy(&env, NoArgs);
        (env, registry)
    }

    fn member(env: &HostEnv) -> MockDappContractHostRef {
        MockDappContract::deploy(env, NoArgs)
    }

    fn metadata() -> DappMetadata {
        DappMetadata {
            name: "Dapp".to_string(),
            description: "A dapp".to_string(),
            website_url: "https://dapp.example".to_string(),
            icon_url: "https://dapp.example/icon.png".to_string()
        }
    }

    #[test]
    fn init() {
        let (env, registry) = setup();
        let address = registry.address();
        assert_eq!(registry.get_owner(), env.get_account(0));
        assert_eq!(registry.get_dapp_contracts(), vec![address]);
        assert!(registry.is_dapp_contract(&address));
        assert_eq!(registry.get_dapp_metadata(), DappMetadata::default());
    }

    #[test]
    fn set_metadata() {
        let (env, mut registry) = setup();
        registry.set_dapp_metadata(metadata());
        assert_eq!(registry.get_dapp_metadata(), metadata());
        assert!(env.emitted_event(
            &registry,
            DappMetadataChanged {
                name: "Dapp".to_string(),
                description: "A dapp".to_string(),
                website_url: "https://dapp.example".to_string(),
                icon_url: "https://dapp.example/icon.png".to_string()
            }
        ));

        env.set_caller(env.get_account(1));
        assert_eq!(
            registry.try_set_dapp_metadata(DappMetadata::default()),
            Err(AccessError::CallerNotTheOwner.into())
        );
    }

    #[test]
    fn owner_adds_contract() {
        let (env, mut registry) = setup();
        let member = member(&env);
        registry.add_dapp_contract(&member.address(), false);

        assert!(registry.is_dapp_contract(&member.address()));
        assert!(!registry.is_dapp_factory(&member.address()));
        assert_eq!(
            registry.get_dapp_contracts(),
            vec![registry.address(), member.address()]
        );
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: member.address(),
                is_factory: false,
                registrar: env.get_account(0)
            }
        ));
    }

    #[test]
    fn cannot_add_account() {
        let (env, mut registry) = setup();
        assert_eq!(
            registry.try_add_dapp_contract(&env.get_account(1), false),
            Err(Error::NotAContract.into())
        );
    }

    #[test]
    fn adds_any_contract() {
        // The registry does not call the contracts it adds: a contract that knows nothing about dapps
        // can be added, and a contract can belong to several dapps.
        let (env, mut registry) = setup();
        let mut other = OwnedDappRegistry::deploy(&env, NoArgs);
        let unaware = OwnedDappRegistry::deploy(&env, NoArgs).address();
        registry.add_dapp_contract(&unaware, false);
        other.add_dapp_contract(&unaware, false);
        assert!(registry.is_dapp_contract(&unaware));
        assert!(other.is_dapp_contract(&unaware));
        // Registries can list each other.
        registry.add_dapp_contract(&other.address(), false);
        assert_eq!(
            registry.get_dapp_contracts(),
            vec![registry.address(), unaware, other.address()]
        );
    }

    #[test]
    fn cannot_add_twice() {
        let (env, mut registry) = setup();
        let member = member(&env);
        registry.add_dapp_contract(&member.address(), false);
        assert_eq!(
            registry.try_add_dapp_contract(&member.address(), true),
            Err(Error::DappContractAlreadyRegistered.into())
        );
        let address = registry.address();
        assert_eq!(
            registry.try_add_dapp_contract(&address, false),
            Err(Error::DappContractAlreadyRegistered.into())
        );
    }

    #[test]
    fn non_owner_cannot_add() {
        let (env, mut registry) = setup();
        let member = member(&env);
        env.set_caller(env.get_account(1));
        assert_eq!(
            registry.try_add_dapp_contract(&member.address(), false),
            Err(AccessError::CallerNotTheOwner.into())
        );
    }

    #[test]
    fn factory_adds_contracts() {
        let (env, mut registry) = setup();
        let mut factory = member(&env);
        let child = member(&env);
        registry.add_dapp_contract(&factory.address(), true);
        assert!(registry.is_dapp_factory(&factory.address()));

        factory.register_in_dapp(&registry.address(), &child.address(), false);
        assert!(registry.is_dapp_contract(&child.address()));
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: child.address(),
                is_factory: false,
                registrar: factory.address()
            }
        ));

        // A factory cannot register another factory.
        let other = member(&env);
        assert_eq!(
            factory.try_register_in_dapp(&registry.address(), &other.address(), true),
            Err(AccessError::CallerNotTheOwner.into())
        );

        // A contract that is not a registered factory cannot register anything.
        let mut not_factory = member(&env);
        assert_eq!(
            not_factory.try_register_in_dapp(&registry.address(), &other.address(), false),
            Err(Error::CallerNotDappFactory.into())
        );
        registry.add_dapp_contract(&not_factory.address(), false);
        assert_eq!(
            not_factory.try_register_in_dapp(&registry.address(), &other.address(), false),
            Err(Error::CallerNotDappFactory.into())
        );
    }

    #[test]
    fn remove_contracts() {
        let (env, mut registry) = setup();
        let reg = registry.address();
        let a = member(&env).address();
        let b = member(&env).address();
        let c = member(&env).address();
        registry.add_dapp_contract(&a, true);
        registry.add_dapp_contract(&b, false);
        registry.add_dapp_contract(&c, false);

        // The last contract takes the place of the removed one.
        registry.remove_dapp_contract(&a);
        assert_eq!(registry.get_dapp_contracts(), vec![reg, c, b]);
        assert!(!registry.is_dapp_contract(&a));
        assert!(!registry.is_dapp_factory(&a));
        assert!(env.emitted_event(
            &registry,
            DappContractRemoved {
                contract: a,
                registrar: env.get_account(0)
            }
        ));

        // Removing the last contract.
        registry.remove_dapp_contract(&b);
        assert_eq!(registry.get_dapp_contracts(), vec![reg, c]);

        // A removed contract can be added again.
        registry.add_dapp_contract(&a, false);
        assert_eq!(registry.get_dapp_contracts(), vec![reg, c, a]);
        assert!(!registry.is_dapp_factory(&a));
        registry.remove_dapp_contract(&c);
        assert_eq!(registry.get_dapp_contracts(), vec![reg, a]);
        assert!(registry.is_dapp_contract(&a));

        assert_eq!(
            registry.try_remove_dapp_contract(&b),
            Err(Error::DappContractNotRegistered.into())
        );
        assert_eq!(
            registry.try_remove_dapp_contract(&reg),
            Err(Error::CannotRemoveDappRegistry.into())
        );
        env.set_caller(env.get_account(1));
        assert_eq!(
            registry.try_remove_dapp_contract(&a),
            Err(AccessError::CallerNotTheOwner.into())
        );
    }

    #[test]
    fn transfer_ownership() {
        let (env, mut registry) = setup();
        let (owner, new_owner) = (env.get_account(0), env.get_account(1));
        registry.transfer_ownership(&new_owner);
        assert_eq!(registry.get_owner(), new_owner);

        env.set_caller(new_owner);
        registry.set_dapp_metadata(metadata());
        env.set_caller(owner);
        assert_eq!(
            registry.try_set_dapp_metadata(DappMetadata::default()),
            Err(AccessError::CallerNotTheOwner.into())
        );
    }

    #[test]
    fn data_is_stored_under_named_keys() {
        use crate::dapp::storage::{DAPP_CONTRACTS_COUNT_KEY, DAPP_CONTRACTS_KEY, DAPP_NAME_KEY};
        use odra::casper_types::bytesrepr::{FromBytes, ToBytes};
        use odra::schema::{resolve_storage, SchemaStorageLayout, StorageLocation};

        let (env, mut registry) = setup();
        let address = registry.address();
        registry.set_dapp_metadata(metadata());
        let member = member(&env).address();
        registry.add_dapp_contract(&member, true);

        fn named<T: FromBytes>(env: &HostEnv, address: &Address, name: &str) -> T {
            let bytes = env
                .get_named_value(address, name)
                .unwrap_or_else(|| panic!("{name} should be stored"));
            T::from_bytes(&bytes).unwrap().0
        }
        assert_eq!(named::<String>(&env, &address, DAPP_NAME_KEY), "Dapp");
        assert_eq!(named::<u32>(&env, &address, DAPP_CONTRACTS_COUNT_KEY), 1);

        // The contracts by index, under a dictionary with plain string keys.
        let bytes = env
            .get_dictionary_value(&address, DAPP_CONTRACTS_KEY, b"0")
            .expect("the first contract should be stored");
        assert_eq!(Address::from_bytes(&bytes).unwrap().0, member);

        // The dictionaries keyed by address, found through the storage layout.
        let layout = OwnedDappRegistry::storage_kind();
        for (path, expected) in [
            ("dapp.indices", 1u32.to_bytes()),
            ("dapp.factories", true.to_bytes())
        ] {
            let key = member.to_bytes().unwrap();
            let query = resolve_storage(&layout, path, &[key]).unwrap();
            let StorageLocation::Dictionary { name, key } = &query.location else {
                panic!("{path} should be a dictionary")
            };
            let bytes = env
                .get_dictionary_value(&address, name, key.as_bytes())
                .unwrap_or_else(|| panic!("{path} should be stored"));
            assert_eq!(bytes.to_vec(), expected.unwrap());
        }
    }

    #[test]
    fn member_recognizes_registries() {
        let (env, registry) = setup();
        let other = OwnedDappRegistry::deploy(&env, NoArgs);
        let mut member = member(&env);
        assert!(!member.recognizes_dapp_registry(&registry.address()));

        // A member may recognize several registries.
        member.recognize_dapp_registry(&registry.address());
        member.recognize_dapp_registry(&other.address());
        assert!(member.recognizes_dapp_registry(&registry.address()));
        assert!(member.recognizes_dapp_registry(&other.address()));
        assert!(env.emitted_event(
            &member,
            DappRegistryRecognized {
                registry: registry.address()
            }
        ));

        member.forget_dapp_registry(&other.address());
        assert!(!member.recognizes_dapp_registry(&other.address()));
        assert!(env.emitted_event(
            &member,
            DappRegistryForgotten {
                registry: other.address()
            }
        ));
        // Recognizing twice or forgetting an unknown registry changes nothing.
        let events = env.events_count(&member);
        member.recognize_dapp_registry(&registry.address());
        member.forget_dapp_registry(&other.address());
        assert_eq!(env.events_count(&member), events);

        assert_eq!(
            member.try_recognize_dapp_registry(&env.get_account(1)),
            Err(Error::NotAContract.into())
        );
        env.set_caller(env.get_account(1));
        assert_eq!(
            member.try_recognize_dapp_registry(&registry.address()),
            Err(AccessError::CallerNotTheOwner.into())
        );
    }

    #[test]
    fn member_registries_are_stored_under_named_keys() {
        use crate::dapp::storage::DAPP_REGISTRIES_KEY;
        use odra::casper_types::bytesrepr::ToBytes;
        use odra::schema::{resolve_storage, SchemaStorageLayout, StorageLocation};

        let (env, registry) = setup();
        let mut member = member(&env);
        member.recognize_dapp_registry(&registry.address());

        let layout = MockDappContract::storage_kind();
        let key = registry.address().to_bytes().unwrap();
        let query = resolve_storage(&layout, "dapp.registries", &[key]).unwrap();
        let StorageLocation::Dictionary { name, key } = &query.location else {
            panic!("the registries should be a dictionary")
        };
        assert_eq!(name, DAPP_REGISTRIES_KEY);
        let bytes = env
            .get_dictionary_value(&member.address(), name, key.as_bytes())
            .expect("the registry should be stored");
        assert_eq!(bytes.to_vec(), true.to_bytes().unwrap());
    }
}
