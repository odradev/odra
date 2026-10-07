//! A registry grouping the contracts of a dapp.
use super::errors::Error;
use super::events::{DappContractAdded, DappContractRemoved, DappMetadataChanged};
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
    /// Adds a contract to the dapp. A factory can register other contracts.
    ///
    /// The registry calls `get_dapp_registry` on the contract to verify it points back to the registry.
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
/// The read-only functions are entry points. The functions that change state live outside the
/// `#[odra::module]` impl block and do not check who calls them: compose this module with an access module
/// and guard them, as [`OwnedDappRegistry`](super::OwnedDappRegistry) does.
#[odra::module(
    events = [DappMetadataChanged, DappContractAdded, DappContractRemoved],
    errors = Error
)]
pub struct DappRegistryBase {
    metadata: Var<DappMetadata>,
    contracts: List<Address>,
    /// The index of a contract in `contracts` plus one; zero means not registered.
    indices: Mapping<Address, u32>,
    factories: Mapping<Address, bool>
}

#[odra::module]
impl DappRegistryBase {
    /// Returns the dapp metadata, or empty metadata if it has not been set.
    pub fn get_dapp_metadata(&self) -> DappMetadata {
        self.metadata.get_or_default()
    }

    /// Returns the contracts of the dapp, the registry first.
    pub fn get_dapp_contracts(&self) -> Vec<Address> {
        let mut result = vec![self.env().self_address()];
        for address in self.contracts.iter() {
            result.push(address);
        }
        result
    }

    /// Returns true if the contract belongs to the dapp. The registry itself always does.
    pub fn is_dapp_contract(&self, dapp_contract: &Address) -> bool {
        *dapp_contract == self.env().self_address()
            || self.indices.get_or_default(dapp_contract) > 0
    }

    /// Returns true if the contract is a registered factory.
    pub fn is_dapp_factory(&self, dapp_contract: &Address) -> bool {
        self.factories.get_or_default(dapp_contract)
    }

    /// Returns the registry's own address, so the registry answers the
    /// [`DappContract`](super::DappContract) interface like every other contract of the dapp.
    pub fn get_dapp_registry(&self) -> Address {
        self.env().self_address()
    }
}

impl DappRegistryBase {
    /// Sets the dapp metadata and emits [`DappMetadataChanged`].
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn set_dapp_metadata(&mut self, metadata: DappMetadata) {
        self.metadata.set(metadata.clone());
        let DappMetadata {
            name,
            description,
            website_url,
            icon_url
        } = metadata;
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
    pub fn add_dapp_contract(&mut self, dapp_contract: &Address, is_factory: bool) {
        if !dapp_contract.is_contract() {
            self.env().revert(Error::NotAContract);
        }
        if self.is_dapp_contract(dapp_contract) {
            self.env().revert(Error::DappContractAlreadyRegistered);
        }
        // Write first and call out last: a re-entrant add of the same contract is rejected,
        // and a revert of the verification rolls the writes back.
        self.contracts.push(*dapp_contract);
        self.indices.set(dapp_contract, self.contracts.len());
        self.factories.set(dapp_contract, is_factory);

        let reported = DappContractContractRef::new(self.env(), *dapp_contract).get_dapp_registry();
        if reported != self.env().self_address() {
            self.env().revert(Error::DappRegistryMismatch);
        }

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
        let position = self.indices.get_or_default(dapp_contract);
        if position == 0 {
            self.env().revert(Error::DappContractNotRegistered);
        }
        let last = self.contracts.pop().unwrap_or_revert(&self.env());
        if last != *dapp_contract {
            self.contracts.replace(position - 1, last);
            self.indices.set(&last, position);
        }
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
        assert_eq!(registry.get_dapp_registry(), address);
        assert_eq!(registry.get_dapp_metadata(), DappMetadata::default());
    }
}
