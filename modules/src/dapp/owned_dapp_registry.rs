//! A dapp registry owned by a single account.
use super::{DappMetadata, DappRegistryBase};
use crate::access::errors::Error as AccessError;
use crate::access::Ownable;
use odra::prelude::*;

/// A ready-to-deploy dapp registry built from [`Ownable`] and [`DappRegistryBase`].
///
/// The deployer becomes the owner. The owner sets the metadata and adds or removes contracts. A contract
/// the owner registered as a registrar may add contracts too, but not further registrars.
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
    /// The owner may add any contract. A registrar may add contracts that are not registrars.
    /// Anyone else is rejected with `CallerNotTheOwner`, and an unregistered contract with
    /// `CallerNotDappRegistrar`.
    pub fn add_dapp_contract(&mut self, dapp_contract: &Address, is_registrar: bool) {
        let caller = self.env().caller();
        if self.ownable.get_optional_owner() != Some(caller) {
            if is_registrar || !caller.is_contract() {
                self.env().revert(AccessError::CallerNotTheOwner);
            }
            self.dapp.assert_dapp_registrar(&caller);
        }
        self.dapp.add_dapp_contract(dapp_contract, is_registrar);
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
            /// Returns true if the contract is a registrar: it may add other contracts.
            fn is_dapp_registrar(&self, dapp_contract: &Address) -> bool;
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
        assert!(!registry.is_dapp_registrar(&member.address()));
        assert_eq!(
            registry.get_dapp_contracts(),
            vec![registry.address(), member.address()]
        );
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: member.address(),
                is_registrar: false,
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
    fn registrar_adds_contracts() {
        let (env, mut registry) = setup();
        let mut registrar = member(&env);
        let child = member(&env);
        registry.add_dapp_contract(&registrar.address(), true);
        assert!(registry.is_dapp_registrar(&registrar.address()));

        registrar.register_in_dapp(&registry.address(), &child.address(), false);
        assert!(registry.is_dapp_contract(&child.address()));
        assert!(env.emitted_event(
            &registry,
            DappContractAdded {
                contract: child.address(),
                is_registrar: false,
                registrar: registrar.address()
            }
        ));

        // A registrar cannot add another registrar.
        let other = member(&env);
        assert_eq!(
            registrar.try_register_in_dapp(&registry.address(), &other.address(), true),
            Err(AccessError::CallerNotTheOwner.into())
        );

        // A contract that is not a registrar cannot add anything.
        let mut not_registrar = member(&env);
        assert_eq!(
            not_registrar.try_register_in_dapp(&registry.address(), &other.address(), false),
            Err(Error::CallerNotDappRegistrar.into())
        );
        registry.add_dapp_contract(&not_registrar.address(), false);
        assert_eq!(
            not_registrar.try_register_in_dapp(&registry.address(), &other.address(), false),
            Err(Error::CallerNotDappRegistrar.into())
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
        assert!(!registry.is_dapp_registrar(&a));
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
        assert!(!registry.is_dapp_registrar(&a));
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

    /// Reads the item `key` of the `__dapp` dictionary of the contract at `address`.
    fn dapp_item<T: odra::casper_types::bytesrepr::FromBytes>(
        env: &HostEnv,
        address: &Address,
        key: &str
    ) -> T {
        use crate::dapp::storage::DAPP_KEY;
        let bytes = env
            .get_dictionary_value(address, DAPP_KEY, key.as_bytes())
            .unwrap_or_else(|| panic!("{DAPP_KEY}->{key} should be stored"));
        T::from_bytes(&bytes).unwrap().0
    }

    /// The `__dapp` item key of an address in the map under `prefix`.
    fn address_key(prefix: &str, address: &Address) -> String {
        use base64::prelude::{Engine, BASE64_STANDARD};
        use odra::casper_types::bytesrepr::ToBytes;
        format!(
            "{prefix}{}",
            BASE64_STANDARD.encode(address.to_bytes().unwrap())
        )
    }

    #[test]
    fn data_is_stored_in_the_dapp_dictionary() {
        use crate::dapp::storage::{
            CONTRACTS_COUNT_KEY, CONTRACT_INDICES_PREFIX, DAPP_KEY, NAME_KEY, REGISTRARS_PREFIX
        };
        use odra::casper_types::bytesrepr::ToBytes;
        use odra::schema::{resolve_storage, SchemaStorageLayout, StorageLocation};

        let (env, mut registry) = setup();
        let address = registry.address();
        registry.set_dapp_metadata(metadata());
        let member = member(&env).address();
        registry.add_dapp_contract(&member, true);

        assert_eq!(dapp_item::<String>(&env, &address, NAME_KEY), "Dapp");
        assert_eq!(dapp_item::<u32>(&env, &address, CONTRACTS_COUNT_KEY), 1);
        assert_eq!(dapp_item::<Address>(&env, &address, "contracts->0"), member);
        let indices_key = address_key(CONTRACT_INDICES_PREFIX, &member);
        let registrars_key = address_key(REGISTRARS_PREFIX, &member);
        assert_eq!(dapp_item::<u32>(&env, &address, &indices_key), 1);
        assert!(dapp_item::<bool>(&env, &address, &registrars_key));
        // No other named keys.
        assert_eq!(env.get_named_value(&address, "__dapp_name"), None);

        // The storage layout points to the same items.
        let layout = OwnedDappRegistry::storage_kind();
        let member_key = member.to_bytes().unwrap();
        for (path, keys, expected) in [
            ("dapp.name", vec![], NAME_KEY.to_string()),
            (
                "dapp.contracts",
                vec![0u32.to_string().to_bytes().unwrap()],
                "contracts->0".to_string()
            ),
            ("dapp.indices", vec![member_key.clone()], indices_key),
            ("dapp.registrars", vec![member_key], registrars_key)
        ] {
            let query = resolve_storage(&layout, path, &keys).unwrap();
            assert_eq!(
                query.location,
                StorageLocation::Dictionary {
                    name: DAPP_KEY.to_string(),
                    key: expected
                },
                "{path}"
            );
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
    fn member_registries_are_stored_in_the_dapp_dictionary() {
        use crate::dapp::storage::{DAPP_KEY, REGISTRIES_PREFIX};
        use odra::casper_types::bytesrepr::ToBytes;
        use odra::schema::{resolve_storage, SchemaStorageLayout, StorageLocation};

        let (env, registry) = setup();
        let mut member = member(&env);
        member.recognize_dapp_registry(&registry.address());

        let key = address_key(REGISTRIES_PREFIX, &registry.address());
        assert!(dapp_item::<bool>(&env, &member.address(), &key));

        let layout = MockDappContract::storage_kind();
        let query = resolve_storage(
            &layout,
            "dapp.registries",
            &[registry.address().to_bytes().unwrap()]
        )
        .unwrap();
        assert_eq!(
            query.location,
            StorageLocation::Dictionary {
                name: DAPP_KEY.to_string(),
                key
            }
        );
    }
}
