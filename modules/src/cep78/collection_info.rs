use odra::prelude::*;
use odra::Address;
use odra::Sequence;
use odra::Var;

use super::constants;
use super::error::CEP78Error;

#[odra::module]
pub struct CollectionInfo {
    name: Var<String>,
    symbol: Var<String>,
    total_token_supply: Var<u64>,
    counter: Sequence<u64>,
    installer: Var<Address>
}

impl CollectionInfo {
    pub fn init(
        &mut self,
        name: String,
        symbol: String,
        total_token_supply: u64,
        installer: Address
    ) {
        if total_token_supply == 0 {
            self.env().revert(CEP78Error::CannotInstallWithZeroSupply)
        }

        if total_token_supply > constants::MAX_TOTAL_TOKEN_SUPPLY {
            self.env().revert(CEP78Error::ExceededMaxTotalSupply)
        }

        self.name.set(name);
        self.symbol.set(symbol);
        self.total_token_supply.set(total_token_supply);
        self.installer.set(installer);

        self.counter.next_value();
    }

    #[inline]
    pub fn installer(&self) -> Address {
        self.installer
            .get_or_revert_with(CEP78Error::MissingInstaller)
    }

    #[inline]
    pub fn total_token_supply(&self) -> u64 {
        self.total_token_supply.get_or_default()
    }

    #[inline]
    pub fn increment_number_of_minted_tokens(&mut self) {
        self.counter.next_value();
    }

    #[inline]
    pub fn number_of_minted_tokens(&self) -> u64 {
        self.counter.get_current_value()
    }

    #[inline]
    pub fn collection_name(&self) -> String {
        self.name.get_or_default()
    }

    #[inline]
    pub fn collection_symbol(&self) -> String {
        self.symbol.get_or_default()
    }
}
