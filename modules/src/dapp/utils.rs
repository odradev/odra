//! A dapp contract used in tests.
use super::{DappContractBase, DappRegistryContractRef};
use crate::access::Ownable;
use odra::prelude::*;
use odra::ContractRef;

/// A member contract: the owner may point it to another registry, and it can act as a registrar by
/// adding other contracts to a registry.
#[odra::module]
pub struct MockDappContract {
    ownable: SubModule<Ownable>,
    dapp: SubModule<DappContractBase>
}

#[odra::module]
impl MockDappContract {
    pub fn init(&mut self, registry: Address) {
        let owner = self.env().caller();
        self.ownable.init(owner);
        self.dapp.set_dapp_registry(&registry);
    }

    pub fn set_dapp_registry(&mut self, registry: &Address) {
        self.ownable.assert_owner(&self.env().caller());
        self.dapp.set_dapp_registry(registry);
    }

    delegate! {
        to self.dapp {
            fn get_dapp_registry(&self) -> Address;
        }
        to self.ownable {
            fn get_owner(&self) -> Address;
        }
    }

    /// Adds `dapp_contract` to `registry`, as a registrar would.
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
