//! A registry grouping the contracts of a dapp.
use super::errors::Error;
use super::events::{DappContractAdded, DappContractRemoved, DappMetadataChanged};
use super::storage::{
    DappContractIndicesStorage, DappContractsCountStorage, DappContractsStorage,
    DappDescriptionStorage, DappFactoriesStorage, DappIconUrlStorage, DappNameStorage,
    DappWebsiteUrlStorage
};
use super::DappMetadata;
use odra::prelude::*;

/// The interface of a dapp registry.
///
/// A registry keeps the dapp's metadata and the contracts that belong to the dapp. The registry itself is
/// always part of the dapp. The registry alone decides which contracts belong to it: it does not call
/// them, so any contract can be added, and a contract can belong to several dapps. A contract may confirm
/// the membership from its side through [`DappMember`](super::DappMember).
#[odra::external_contract]
pub trait DappRegistry {
    /// Sets the dapp metadata.
    fn set_dapp_metadata(&mut self, metadata: DappMetadata);
    /// Returns the dapp metadata.
    fn get_dapp_metadata(&self) -> DappMetadata;
    /// Adds a contract to the dapp. A factory can register other contracts.
    fn add_dapp_contract(&mut self, dapp_contract: &Address, is_factory: bool);
    /// Returns the contracts of the dapp, the registry first.
    fn get_dapp_contracts(&self) -> Vec<Address>;
    /// Removes a contract from the dapp.
    fn remove_dapp_contract(&mut self, dapp_contract: &Address);
    /// Returns true if the contract belongs to the dapp.
    fn is_dapp_contract(&self, dapp_contract: &Address) -> bool;
    /// Returns true if the contract is a registered factory.
    fn is_dapp_factory(&self, dapp_contract: &Address) -> bool;
}

/// Storage and logic of a dapp registry.
///
/// The data lives under named keys and dictionaries, see [`storage`](super::storage). The read-only
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
    contracts_count: SubModule<DappContractsCountStorage>,
    contracts: SubModule<DappContractsStorage>,
    indices: SubModule<DappContractIndicesStorage>,
    factories: SubModule<DappFactoriesStorage>
}

#[odra::module]
impl DappRegistryBase {
    /// Creates the dictionaries of the registry.
    pub fn init(&mut self) {
        self.contracts.init();
        self.indices.init();
        self.factories.init();
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
        let count = self.contracts_count();
        let mut result = Vec::with_capacity(count as usize + 1);
        result.push(self.env().self_address());
        for index in 0..count {
            result.push(self.contract_at(index));
        }
        result
    }

    /// Returns true if the contract belongs to the dapp. The registry itself always does.
    pub fn is_dapp_contract(&self, dapp_contract: &Address) -> bool {
        *dapp_contract == self.env().self_address() || self.position(dapp_contract) > 0
    }

    /// Returns true if the contract is a registered factory.
    pub fn is_dapp_factory(&self, dapp_contract: &Address) -> bool {
        self.factories.get(dapp_contract).unwrap_or_default()
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
    /// Reverts with [`Error::NotAContract`] for an account address and with
    /// [`Error::DappContractAlreadyRegistered`] if the contract already belongs to the dapp.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn add_dapp_contract(&mut self, dapp_contract: &Address, is_factory: bool) {
        if !dapp_contract.is_contract() {
            self.env().revert(Error::NotAContract);
        }
        if self.is_dapp_contract(dapp_contract) {
            self.env().revert(Error::DappContractAlreadyRegistered);
        }
        let count = self.contracts_count();
        self.contracts.set(&count.to_string(), *dapp_contract);
        self.contracts_count.set(count + 1);
        self.indices.set(dapp_contract, count + 1);
        self.factories.set(dapp_contract, is_factory);
        self.env().emit_event(DappContractAdded {
            contract: *dapp_contract,
            is_factory,
            registrar: self.env().caller()
        });
    }

    /// Removes a contract from the dapp and emits [`DappContractRemoved`].
    ///
    /// The last contract takes the place of the removed one, so the order of
    /// [`get_dapp_contracts`](Self::get_dapp_contracts) changes. Reverts with
    /// [`Error::CannotRemoveDappRegistry`] for the registry itself and with
    /// [`Error::DappContractNotRegistered`] for a contract that does not belong to the dapp.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn remove_dapp_contract(&mut self, dapp_contract: &Address) {
        if *dapp_contract == self.env().self_address() {
            self.env().revert(Error::CannotRemoveDappRegistry);
        }
        let position = self.position(dapp_contract);
        if position == 0 {
            self.env().revert(Error::DappContractNotRegistered);
        }
        let count = self.contracts_count();
        let last = self.contract_at(count - 1);
        if last != *dapp_contract {
            self.contracts.set(&(position - 1).to_string(), last);
            self.indices.set(&last, position);
        }
        // The entry of the last contract stays behind; the count bounds the list.
        self.contracts_count.set(count - 1);
        self.indices.set(dapp_contract, 0);
        self.factories.set(dapp_contract, false);

        self.env().emit_event(DappContractRemoved {
            contract: *dapp_contract,
            registrar: self.env().caller()
        });
    }

    /// Reverts with [`Error::CallerNotDappFactory`] unless `address` is a registered factory.
    pub fn assert_dapp_factory(&self, address: &Address) {
        if !self.is_dapp_factory(address) {
            self.env().revert(Error::CallerNotDappFactory);
        }
    }

    fn contracts_count(&self) -> u32 {
        self.contracts_count.get().unwrap_or_default()
    }

    fn contract_at(&self, index: u32) -> Address {
        self.contracts
            .get(&index.to_string())
            .unwrap_or_revert(&self.env())
    }

    /// The index of the contract in the list plus one; zero means not registered.
    fn position(&self, dapp_contract: &Address) -> u32 {
        self.indices.get(dapp_contract).unwrap_or_default()
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
        assert!(!registry.is_dapp_factory(&address));
        assert_eq!(registry.get_dapp_metadata(), DappMetadata::default());
    }
}
