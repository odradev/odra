//! A registry grouping the contracts of a dapp.
use super::errors::Error;
use super::events::{DappContractAdded, DappContractRemoved, DappMetadataChanged};
use super::storage::{
    DappContractsStorage, DappDescriptionStorage, DappIconUrlStorage, DappNameStorage,
    DappRegistrarsStorage, DappWebsiteUrlStorage
};
use super::{DappContractContractRef, DappMetadata};
use odra::prelude::*;
use odra::ContractRef;

/// The interface of a dapp registry.
///
/// A registry keeps the dapp's metadata and the contracts that belong to the dapp. The registry itself is
/// always part of the dapp.
#[odra::external_contract]
pub trait DappRegistry {
    /// Sets the dapp metadata.
    fn set_dapp_metadata(&mut self, metadata: DappMetadata);
    /// Returns the dapp metadata.
    fn get_dapp_metadata(&self) -> DappMetadata;
    /// Adds a contract to the dapp. A registrar may add other contracts.
    ///
    /// The registry calls `get_dapp_registry` on the contract to verify it points back to the registry.
    fn add_dapp_contract(&mut self, dapp_contract: &Address, is_registrar: bool);
    /// Returns the contracts of the dapp, the registry first.
    fn get_dapp_contracts(&self) -> Vec<Address>;
    /// Removes a contract from the dapp.
    fn remove_dapp_contract(&mut self, dapp_contract: &Address);
    /// Returns true if the contract belongs to the dapp.
    fn is_dapp_contract(&self, dapp_contract: &Address) -> bool;
    /// Returns true if the contract is a registrar: it may add other contracts.
    fn is_dapp_registrar(&self, dapp_contract: &Address) -> bool;
}

/// Storage and logic of a dapp registry.
///
/// The data lives in the `__dapp` dictionary, see [`storage`](super::storage). The read-only
/// functions are entry points. The functions that change state live outside the `#[odra::module]` impl
/// block and do not check who calls them: compose this module with an access module and guard them, as
/// [`OwnedDappRegistry`](super::OwnedDappRegistry) does, and call [`init`](Self::init) from the composing
/// contract's `init`.
#[odra::module(
    events = [DappMetadataChanged, DappContractAdded, DappContractRemoved],
    errors = Error
)]
pub struct DappRegistryBase {
    name: SubModule<DappNameStorage>,
    description: SubModule<DappDescriptionStorage>,
    website_url: SubModule<DappWebsiteUrlStorage>,
    icon_url: SubModule<DappIconUrlStorage>,
    contracts: SubModule<DappContractsStorage>,
    registrars: SubModule<DappRegistrarsStorage>
}

#[odra::module]
impl DappRegistryBase {
    /// Creates the `__dapp` dictionary holding the registry's data.
    pub fn init(&mut self) {
        self.contracts.init();
    }

    /// Returns the dapp metadata, or empty metadata if it has not been set.
    pub fn get_dapp_metadata(&self) -> DappMetadata {
        DappMetadata {
            name: self.name.get().unwrap_or_default(),
            description: self.description.get().unwrap_or_default(),
            website_url: self.website_url.get().unwrap_or_default(),
            icon_url: self.icon_url.get().unwrap_or_default()
        }
    }

    /// Returns the contracts of the dapp, the registry first.
    pub fn get_dapp_contracts(&self) -> Vec<Address> {
        let mut result = vec![self.env().self_address()];
        result.extend(self.contracts());
        result
    }

    /// Returns true if the contract belongs to the dapp. The registry itself always does.
    pub fn is_dapp_contract(&self, dapp_contract: &Address) -> bool {
        *dapp_contract == self.env().self_address() || self.contracts().contains(dapp_contract)
    }

    /// Returns true if the contract is a registrar: it may add other contracts.
    pub fn is_dapp_registrar(&self, dapp_contract: &Address) -> bool {
        self.registrars().contains(dapp_contract)
    }

    /// Returns the registry's own address, so the registry answers the
    /// [`DappContract`](crate::dapp::DappContract) interface like every other contract of the dapp.
    pub fn get_dapp_registry(&self) -> Address {
        self.env().self_address()
    }
}

impl DappRegistryBase {
    /// Sets the dapp metadata and emits [`DappMetadataChanged`].
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn set_dapp_metadata(&mut self, metadata: DappMetadata) {
        let DappMetadata {
            name,
            description,
            website_url,
            icon_url
        } = metadata;
        self.name.set(name.clone());
        self.description.set(description.clone());
        self.website_url.set(website_url.clone());
        self.icon_url.set(icon_url.clone());
        self.env().emit_event(DappMetadataChanged {
            name,
            description,
            website_url,
            icon_url
        });
    }

    /// Adds a contract to the dapp and emits [`DappContractAdded`].
    ///
    /// Calls `get_dapp_registry` on the contract and reverts with [`Error::DappRegistryMismatch`] unless it
    /// returns this registry. Reverts with [`Error::NotAContract`] for an account address and with
    /// [`Error::DappContractAlreadyRegistered`] if the contract already belongs to the dapp.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn add_dapp_contract(&mut self, dapp_contract: &Address, is_registrar: bool) {
        if !dapp_contract.is_contract() {
            self.env().revert(Error::NotAContract);
        }
        if self.is_dapp_contract(dapp_contract) {
            self.env().revert(Error::DappContractAlreadyRegistered);
        }
        // Write first and call out last: a re-entrant add of the same contract is rejected,
        // and a revert of the verification rolls the writes back.
        let mut contracts = self.contracts();
        contracts.push(*dapp_contract);
        self.contracts.set(contracts);
        if is_registrar {
            let mut registrars = self.registrars();
            registrars.push(*dapp_contract);
            self.registrars.set(registrars);
        }

        let reported = DappContractContractRef::new(self.env(), *dapp_contract).get_dapp_registry();
        if reported != self.env().self_address() {
            self.env().revert(Error::DappRegistryMismatch);
        }

        self.env().emit_event(DappContractAdded {
            contract: *dapp_contract,
            is_registrar,
            registrar: self.env().caller()
        });
    }

    /// Removes a contract from the dapp and emits [`DappContractRemoved`].
    ///
    /// The other contracts keep their order. Reverts with
    /// [`Error::CannotRemoveDappRegistry`] for the registry itself and with
    /// [`Error::DappContractNotRegistered`] for a contract that does not belong to the dapp.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn remove_dapp_contract(&mut self, dapp_contract: &Address) {
        if *dapp_contract == self.env().self_address() {
            self.env().revert(Error::CannotRemoveDappRegistry);
        }
        let mut contracts = self.contracts();
        let Some(position) = contracts.iter().position(|c| c == dapp_contract) else {
            self.env().revert(Error::DappContractNotRegistered)
        };
        contracts.remove(position);
        self.contracts.set(contracts);
        let mut registrars = self.registrars();
        if let Some(position) = registrars.iter().position(|c| c == dapp_contract) {
            registrars.remove(position);
            self.registrars.set(registrars);
        }

        self.env().emit_event(DappContractRemoved {
            contract: *dapp_contract,
            registrar: self.env().caller()
        });
    }

    /// Reverts with [`Error::CallerNotDappRegistrar`] unless `address` is a registrar.
    pub fn assert_dapp_registrar(&self, address: &Address) {
        if !self.is_dapp_registrar(address) {
            self.env().revert(Error::CallerNotDappRegistrar);
        }
    }

    /// The registered contracts, the registry not included.
    fn contracts(&self) -> Vec<Address> {
        self.contracts.get().unwrap_or_default()
    }

    fn registrars(&self) -> Vec<Address> {
        self.registrars.get().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn empty_registry() {
        let env = odra_test::odra_env();
        let registry = DappRegistryBase::deploy(&env, NoArgs);
        let address = registry.address();
        assert_eq!(registry.get_dapp_contracts(), vec![address]);
        assert!(registry.is_dapp_contract(&address));
        assert!(!registry.is_dapp_contract(&env.get_account(0)));
        assert!(!registry.is_dapp_registrar(&address));
        assert_eq!(registry.get_dapp_registry(), address);
        assert_eq!(registry.get_dapp_metadata(), DappMetadata::default());
    }
}
