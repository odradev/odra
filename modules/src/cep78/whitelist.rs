use odra::{args::Maybe, prelude::*, Address, SubModule};

use crate::{key_value_storage, single_value_storage};

use super::{
    constants::{ACL_PACKAGE_MODE, ACL_WHITELIST, WHITELIST_MODE},
    error::CEP78Error,
    modalities::WhitelistMode
};

single_value_storage!(
    Cep78WhitelistMode,
    WhitelistMode,
    WHITELIST_MODE,
    CEP78Error::InvalidACLPackageMode
);
single_value_storage!(
    Cep78PackageMode,
    bool,
    ACL_PACKAGE_MODE,
    CEP78Error::InvalidACLPackageMode
);
key_value_storage!(Cep78ACLWhitelist, ACL_WHITELIST, bool);
impl Cep78ACLWhitelist {
    pub fn clear(&self) {
        self.env().remove_dictionary(ACL_WHITELIST);
    }
}

#[odra::module]
pub struct ACLWhitelist {
    whitelist: SubModule<Cep78ACLWhitelist>,
    mode: SubModule<Cep78WhitelistMode>,
    package_mode: SubModule<Cep78PackageMode>
}

impl ACLWhitelist {
    pub fn init(&mut self, addresses: Vec<Address>, mode: WhitelistMode) {
        for address in addresses.iter() {
            self.whitelist.set(&address.to_string(), true);
        }
        self.mode.set(mode);
        // Odra does not support version mode.
        self.package_mode.set(true);
    }

    #[inline]
    pub fn get_mode(&self) -> WhitelistMode {
        self.mode.get()
    }

    #[inline]
    pub fn is_whitelisted(&self, address: &Address) -> bool {
        self.whitelist.get(&address.to_string()).unwrap_or_default()
    }

    #[inline]
    pub fn get_package_mode(&self) -> bool {
        self.package_mode.get()
    }

    pub fn update(&mut self, new_addresses: Maybe<Vec<Address>>) {
        let new_addresses = new_addresses.unwrap_or_default();
        if !new_addresses.is_empty() {
            match self.get_mode() {
                WhitelistMode::Unlocked => {
                    self.whitelist.clear();
                    for address in new_addresses.iter() {
                        self.whitelist.set(&address.to_string(), true);
                    }
                }
                WhitelistMode::Locked => self.env().revert(CEP78Error::InvalidWhitelistMode)
            }
        }
    }
}
