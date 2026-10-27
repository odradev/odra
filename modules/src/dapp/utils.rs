//! A dapp contract used in tests.
use super::{DappMemberBase, DappRegistryContractRef};
use crate::access::Ownable;
use odra::prelude::*;
use odra::ContractRef;

/// A contract that recognizes the registries its owner picks, and can act as a registrar by adding
/// other contracts in a registry.
#[odra::module]
pub struct MockDappContract {
    ownable: SubModule<Ownable>,
    dapp: SubModule<DappMemberBase>
}

#[odra::module]
impl MockDappContract {
    pub fn init(&mut self) {
        let owner = self.env().caller();
        self.ownable.init(owner);
        self.dapp.init();
    }

    pub fn recognize_dapp_registry(&mut self, registry: &Address) {
        self.ownable.assert_owner(&self.env().caller());
        self.dapp.recognize_dapp_registry(registry);
    }

    pub fn forget_dapp_registry(&mut self, registry: &Address) {
        self.ownable.assert_owner(&self.env().caller());
        self.dapp.forget_dapp_registry(registry);
    }

    delegate! {
        to self.dapp {
            fn recognizes_dapp_registry(&self, registry: &Address) -> bool;
        }
        to self.ownable {
            fn get_owner(&self) -> Address;
        }
    }

    /// Registers `dapp_contract` in `registry`, as a registrar would.
    pub fn register_in_dapp(
        &mut self,
        registry: &Address,
        dapp_contract: &Address,
        is_registrar: bool
    ) {
        DappRegistryContractRef::new(self.env(), *registry)
            .add_dapp_contract(dapp_contract, is_registrar);
    }
}
