use odra::{args::Maybe, prelude::*, Address, Var};

use super::{error::CEP78Error, modalities::WhitelistMode};

#[odra::module]
pub struct ACLWhitelist {
    addresses: Var<Vec<Address>>,
    mode: Var<WhitelistMode>,
    package_mode: Var<bool>
}

impl ACLWhitelist {
    pub fn init(&mut self, addresses: Vec<Address>, mode: WhitelistMode) {
        self.addresses.set(addresses);
        self.mode.set(mode);
        // Odra does not support version mode.
        self.package_mode.set(true);
    }

    #[inline]
    pub fn get_mode(&self) -> WhitelistMode {
        self.mode.get_or_default()
    }

    #[inline]
    pub fn is_whitelisted(&self, address: &Address) -> bool {
        self.addresses.get_or_default().contains(address)
    }

    pub fn update(&mut self, new_addresses: Maybe<Vec<Address>>) {
        let new_addresses = new_addresses.unwrap_or_default();
        if !new_addresses.is_empty() {
            match self.get_mode() {
                WhitelistMode::Unlocked => {
                    self.addresses.set(new_addresses);
                }
                WhitelistMode::Locked => self.env().revert(CEP78Error::InvalidWhitelistMode)
            }
        }
    }
}
