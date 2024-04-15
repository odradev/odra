use odra::{prelude::*, Mapping, UnwrapOrRevert, Var};

use super::{
    error::CEP78Error,
    modalities::{OwnerReverseLookupMode, TokenIdentifier},
};

#[odra::module]
pub struct ReverseLookup {
    hash_by_index: Mapping<u64, String>,
    index_by_hash: Mapping<String, u64>,
    mode: Var<OwnerReverseLookupMode>
}

impl ReverseLookup {
    pub fn init(&mut self, mode: OwnerReverseLookupMode) {
        self.mode.set(mode);
    }

    #[inline]
    pub fn get_mode(&self) -> OwnerReverseLookupMode {
        self.mode.get_or_default()
    }

    pub fn insert_hash(
        &mut self,
        current_number_of_minted_tokens: u64,
        token_identifier: &TokenIdentifier
    ) {
        if token_identifier.get_index().is_some() {
            return;
        }
        if self
            .index_by_hash
            .get(&token_identifier.to_string())
            .is_some()
        {
            self.env().revert(CEP78Error::DuplicateIdentifier)
        }
        if self
            .hash_by_index
            .get(&current_number_of_minted_tokens)
            .is_some()
        {
            self.env().revert(CEP78Error::DuplicateIdentifier)
        }

        self.hash_by_index.set(
            &current_number_of_minted_tokens,
            token_identifier.get_hash().unwrap_or_revert(&self.env())
        );
        self.index_by_hash.set(
            &token_identifier.to_string(),
            current_number_of_minted_tokens
        );
    }

    pub fn get_token_index(&self, token_identifier: &TokenIdentifier) -> u64 {
        match token_identifier {
            TokenIdentifier::Index(token_index) => *token_index,
            TokenIdentifier::Hash(_) => self
                .index_by_hash
                .get(&token_identifier.to_string())
                .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidTokenIdentifier)
        }
    }

    // pub fn remove(&mut self, index: u64, hash: String) {
    //     self.hash_by_index.remove(&index.to_string());
    //     self.index_by_hash.remove(&hash);
    // }

    // pub fn get_by_index(&self, index: u64) -> Option<Address> {
    //     self.hash_by_index.get(&index.to_string()).copied()
    // }

    // pub fn get_by_hash(&self, hash: &str) -> Option<Address> {
    //     self.index_by_hash.get(hash).copied()
    // }
}
