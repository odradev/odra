use odra::{args::Maybe, casper_types::{ContractHash, Key}, prelude::*, Address, List, UnwrapOrRevert, Var};

use super::{error::CEP78Error, modalities::WhitelistMode, utils::GetAs};

#[odra::module]
pub struct ACLWhitelist {
    addresses: List<Address>,
    mode: Var<u8>,
    package_mode: Var<bool>,
}

impl ACLWhitelist {
    pub fn init(&mut self, addresses: Vec<Address>, mode: u8, package_mode: bool) {
        for address in addresses {
            self.addresses.push(address);
        }
        self.mode.set(mode);
        self.package_mode.set(package_mode);
    }

    pub fn get_mode(&self) -> WhitelistMode {
        self.mode.get_as(&self.env())
    }

    pub fn is_package_mode(&self) -> bool {
        self.package_mode.get_or_default()
    }

    pub fn update_package_mode(&mut self, package_mode: Maybe<bool>) {
        if let Maybe::Some(package_mode) = package_mode {
            self.package_mode.set(package_mode);
        }
    }

    pub fn update_addresses(&mut self, addresses: Maybe<Vec<Address>>, contract_whitelist: Maybe<Vec<ContractHash>>) {
        let mut new_addresses = addresses.unwrap_or_default();
    
        // Deprecated in 1.4 in favor of above ARG_ACL_WHITELIST
        let new_contract_whitelist = contract_whitelist.unwrap_or_default();
    
        for contract_hash in new_contract_whitelist.iter() {
            let address = Address::try_from(Key::from(*contract_hash)).unwrap_or_revert(&self.env());
            new_addresses.push(address);
        }
    
        if !new_addresses.is_empty() {
            match self.get_mode() {
                WhitelistMode::Unlocked => {
                    while let Some(_) = self.addresses.pop() {
                        
                    }
                    for address in new_addresses {
                        self.addresses.push(address);
                    }
                }
                WhitelistMode::Locked => self.env().revert(CEP78Error::InvalidWhitelistMode),
            }
        }
    }
}