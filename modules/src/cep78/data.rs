use odra::prelude::*;
use odra::Address;
use odra::Mapping;
use odra::UnwrapOrRevert;
use odra::Var;

use super::constants;
use super::error::CEP78Error;

#[odra::module]
pub struct CollectionData {
    name: Var<String>,
    symbol: Var<String>,
    total_token_supply: Var<u64>,
    counter: Var<u64>,
    installer: Var<Address>,
    owners: Mapping<String, Address>,
    issuers: Mapping<String, Address>,
    approved: Mapping<String, Option<Address>>,
    token_count: Mapping<Address, u64>,
    burnt_tokens: Mapping<String, ()>,
    operators: Mapping<(Address, Address), bool>
}

impl CollectionData {
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
        self.counter.add(1);
    }

    #[inline]
    pub fn number_of_minted_tokens(&self) -> u64 {
        self.counter.get_or_default()
    }

    #[inline]
    pub fn collection_name(&self) -> String {
        self.name.get_or_default()
    }

    #[inline]
    pub fn collection_symbol(&self) -> String {
        self.symbol.get_or_default()
    }

    #[inline]
    pub fn set_owner(&mut self, token_id: &String, token_owner: Address) {
        self.owners.set(token_id, token_owner);
    }

    #[inline]
    pub fn set_issuer(&mut self, token_id: &String, issuer: Address) {
        self.issuers.set(token_id, issuer);
    }

    #[inline]
    pub fn increment_counter(&mut self, token_owner: &Address) {
        self.token_count.add(token_owner, 1);
    }

    #[inline]
    pub fn decrement_counter(&mut self, token_owner: &Address) {
        self.token_count.subtract(token_owner, 1);
    }

    #[inline]
    pub fn operator(&self, owner: Address, operator: Address) -> bool {
        self.operators.get_or_default(&(owner, operator))
    }

    #[inline]
    pub fn set_operator(&mut self, owner: Address, operator: Address, approved: bool) {
        self.operators.set(&(owner, operator), approved);
    }

    #[inline]
    pub fn mark_burnt(&mut self, token_id: &String) {
        self.burnt_tokens.set(token_id, ());
    }

    #[inline]
    pub fn is_burnt(&self, token_id: &String) -> bool {
        self.burnt_tokens.get(token_id).is_some()
    }

    #[inline]
    pub fn issuer(&self, token_id: &String) -> Address {
        self.issuers
            .get(token_id)
            .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidTokenIdentifier)
    }

    #[inline]
    pub fn approve(&mut self, token_id: &String, operator: Address) {
        self.approved.set(token_id, Some(operator));
    }

    #[inline]
    pub fn revoke(&mut self, token_id: &String) {
        self.approved.set(token_id, None);
    }

    #[inline]
    pub fn approved(&self, token_id: &String) -> Option<Address> {
        self.approved.get(token_id).flatten()
    }

    #[inline]
    pub fn owner_of(&self, token_id: &String) -> Option<Address> {
        self.owners.get(token_id)
    }

    #[inline]
    pub fn token_count(&self, owner: &Address) -> u64 {
        self.token_count.get(owner).unwrap_or_default()
    }
}
