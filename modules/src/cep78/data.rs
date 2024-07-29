use odra::named_keys::{
    base64_encoded_key_value_storage, compound_key_value_storage, key_value_storage,
    single_value_storage
};
use odra::{casper_types::bytesrepr::ToBytes, prelude::*, Address, SubModule, UnwrapOrRevert};

use super::constants::*;
use super::error::CEP78Error;

single_value_storage!(
    Cep78CollectionName,
    String,
    COLLECTION_NAME,
    CEP78Error::MissingCollectionName
);
single_value_storage!(
    Cep78CollectionSymbol,
    String,
    COLLECTION_SYMBOL,
    CEP78Error::MissingCollectionName
);
single_value_storage!(
    Cep78TotalSupply,
    u64,
    TOTAL_TOKEN_SUPPLY,
    CEP78Error::MissingTotalTokenSupply
);
single_value_storage!(Cep78TokenCounter, u64, NUMBER_OF_MINTED_TOKENS);
impl Cep78TokenCounter {
    pub fn add(&mut self, value: u64) {
        match self.get() {
            Some(current_value) => self.set(value + current_value),
            None => self.set(value)
        }
    }

    pub fn sub(&mut self, value: u64) {
        match self.get() {
            Some(current_value) => self.set(current_value - value),
            None => self.env().revert(CEP78Error::GoingBelowZeroSupply)
        }
    }
}
single_value_storage!(
    Cep78Installer,
    Address,
    INSTALLER,
    CEP78Error::MissingInstaller
);
compound_key_value_storage!(Cep78Operators, OPERATORS, Address, bool);
key_value_storage!(Cep78Owners, TOKEN_OWNERS, Address);
key_value_storage!(Cep78Issuers, TOKEN_ISSUERS, Address);
key_value_storage!(Cep78BurntTokens, BURNT_TOKENS, bool);
base64_encoded_key_value_storage!(Cep78TokenCount, TOKEN_COUNT, Address, u64);
key_value_storage!(Cep78Approved, APPROVED, Option<Address>);

#[odra::module]
pub struct CollectionData {
    name: SubModule<Cep78CollectionName>,
    symbol: SubModule<Cep78CollectionSymbol>,
    total_token_supply: SubModule<Cep78TotalSupply>,
    minted_tokens_count: SubModule<Cep78TokenCounter>,
    installer: SubModule<Cep78Installer>,
    owners: SubModule<Cep78Owners>,
    issuers: SubModule<Cep78Issuers>,
    approved: SubModule<Cep78Approved>,
    token_count: SubModule<Cep78TokenCount>,
    burnt_tokens: SubModule<Cep78BurntTokens>,
    operators: SubModule<Cep78Operators>
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

        if total_token_supply > MAX_TOTAL_TOKEN_SUPPLY {
            self.env().revert(CEP78Error::ExceededMaxTotalSupply)
        }

        self.name.set(name);
        self.symbol.set(symbol);
        self.total_token_supply.set(total_token_supply);
        self.installer.set(installer);
    }

    #[inline]
    pub fn installer(&self) -> Address {
        self.installer.get()
    }

    #[inline]
    pub fn total_token_supply(&self) -> u64 {
        self.total_token_supply.get()
    }

    #[inline]
    pub fn increment_number_of_minted_tokens(&mut self) {
        self.minted_tokens_count.add(1);
    }

    #[inline]
    pub fn decrement_number_of_minted_tokens(&mut self) {
        self.minted_tokens_count.sub(1);
    }

    #[inline]
    pub fn number_of_minted_tokens(&self) -> u64 {
        self.minted_tokens_count.get().unwrap_or_default()
    }

    #[inline]
    pub fn collection_name(&self) -> String {
        self.name.get()
    }

    #[inline]
    pub fn collection_symbol(&self) -> String {
        self.symbol.get()
    }

    #[inline]
    pub fn set_owner(&mut self, token_id: &str, token_owner: Address) {
        self.owners.set(token_id, token_owner);
    }

    #[inline]
    pub fn set_issuer(&mut self, token_id: &str, issuer: Address) {
        self.issuers.set(token_id, issuer);
    }

    #[inline]
    pub fn increment_counter(&mut self, token_owner: &Address) {
        let value = self.token_count.get(token_owner).unwrap_or_default();
        self.token_count.set(token_owner, value + 1);
    }

    #[inline]
    pub fn decrement_counter(&mut self, token_owner: &Address) {
        let value = self.token_count.get(token_owner).unwrap_or_default();
        self.token_count.set(token_owner, value - 1);
    }

    #[inline]
    pub fn operator(&self, owner: Address, operator: Address) -> bool {
        self.operators.get_or_default(&owner, &operator)
    }

    #[inline]
    pub fn set_operator(&mut self, owner: Address, operator: Address, approved: bool) {
        self.operators.set(&owner, &operator, approved);
    }

    #[inline]
    pub fn mark_burnt(&mut self, token_id: &str) {
        self.burnt_tokens.set(token_id, true);
    }

    #[inline]
    pub fn mark_not_burnt(&mut self, token_id: &str) {
        self.burnt_tokens.set(token_id, false);
    }

    #[inline]
    pub fn is_burnt(&self, token_id: &str) -> bool {
        self.burnt_tokens.get(token_id).unwrap_or_default()
    }

    #[inline]
    pub fn issuer(&self, token_id: &str) -> Address {
        self.issuers
            .get(token_id)
            .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidTokenIdentifier)
    }

    #[inline]
    pub fn approve(&mut self, token_id: &str, operator: Address) {
        self.approved.set(token_id, Some(operator));
    }

    #[inline]
    pub fn revoke(&mut self, token_id: &str) {
        self.approved.set(token_id, None);
    }

    #[inline]
    pub fn approved(&self, token_id: &str) -> Option<Address> {
        self.approved.get(token_id).flatten()
    }

    #[inline]
    pub fn owner_of(&self, token_id: &str) -> Option<Address> {
        self.owners.get(token_id)
    }

    #[inline]
    pub fn token_count(&self, owner: &Address) -> u64 {
        self.token_count.get(owner).unwrap_or_default()
    }
}
