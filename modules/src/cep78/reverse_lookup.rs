use odra::{
    args::Maybe,
    named_keys::{key_value_storage, single_value_storage},
    prelude::*,
    Address, Mapping, Sequence, SubModule, UnwrapOrRevert
};

use super::{
    constants::{PAGE_LIMIT, PAGE_TABLE, PREFIX_PAGE_DICTIONARY, RECEIPT_NAME, REPORTING_MODE},
    error::CEP78Error,
    modalities::{OwnerReverseLookupMode, OwnershipMode, TokenIdentifier},
    utils
};
// The size of a given page, it is currently set to 1000
// to ease the math around addressing newly minted tokens.
pub const PAGE_SIZE: u64 = 1000;

single_value_storage!(
    Cep78OwnerReverseLookupMode,
    OwnerReverseLookupMode,
    REPORTING_MODE,
    CEP78Error::InvalidReportingMode
);
single_value_storage!(
    Cep78ReceiptName,
    String,
    RECEIPT_NAME,
    CEP78Error::InvalidReceiptName
);
single_value_storage!(
    Cep78PageLimit,
    u64,
    PAGE_LIMIT,
    CEP78Error::InvalidPageLimit
);
key_value_storage!(Cep78PageTable, PAGE_TABLE, Vec<bool>);

#[odra::module]
pub struct ReverseLookup {
    mode: SubModule<Cep78OwnerReverseLookupMode>,
    hash_by_index: Mapping<u64, String>,
    index_by_hash: Mapping<String, u64>,
    page_table: SubModule<Cep78PageTable>,
    receipt_name: SubModule<Cep78ReceiptName>,
    page_limit: SubModule<Cep78PageLimit>,
    tokens_count: Sequence<u64>
}

impl ReverseLookup {
    pub fn init(&mut self, mode: OwnerReverseLookupMode, receipt_name: String, tokens: u64) {
        self.mode.set(mode);
        self.receipt_name.set(receipt_name);

        if [
            OwnerReverseLookupMode::Complete,
            OwnerReverseLookupMode::TransfersOnly
        ]
        .contains(&mode)
        {
            let page_table_width = utils::max_number_of_pages(tokens);
            self.page_limit.set(page_table_width);
        }
    }

    pub fn register_owner(&mut self, owner: Maybe<Address>, ownership_mode: OwnershipMode) {
        let mode = self.get_mode();
        if [
            OwnerReverseLookupMode::Complete,
            OwnerReverseLookupMode::TransfersOnly
        ]
        .contains(&mode)
        {
            let env = self.env();
            let owner = match ownership_mode {
                OwnershipMode::Minter => env.caller(),
                OwnershipMode::Assigned | OwnershipMode::Transferable => owner.unwrap(&env)
            };
            let owner_key = utils::address_to_key(&owner);
            if self.page_table.get(&owner_key).is_none() {
                let page_limit = self.page_limit.get();
                self.page_table
                    .set(&owner_key, vec![false; page_limit as usize]);
            }
        }
    }

    pub fn on_mint(&mut self, token_owner: &Address, token_identifier: &TokenIdentifier) {
        if self.get_mode() == OwnerReverseLookupMode::Complete {
            let token_index = self.prepare_token_index(token_identifier);
            self.update_page(
                token_index,
                token_owner,
                CEP78Error::UnregisteredOwnerInMint,
                true
            );
        }
    }

    pub fn on_transfer(
        &mut self,
        token_identifier: &TokenIdentifier,
        source: &Address,
        target: &Address
    ) {
        use OwnerReverseLookupMode::*;
        if matches!(self.get_mode(), Complete | TransfersOnly) {
            let token_index = self.get_token_index_checked(token_identifier);

            self.update_page(
                token_index,
                source,
                CEP78Error::UnregisteredOwnerInTransfer,
                false
            );
            self.update_page(
                token_index,
                target,
                CEP78Error::UnregisteredOwnerInTransfer,
                true
            );
        }
    }

    pub fn on_burn(&mut self, token_owner: &Address, token_identifier: &TokenIdentifier) {
        if self.get_mode() == OwnerReverseLookupMode::Complete {
            let token_index = self.get_token_index_checked(token_identifier);
            self.update_page(
                token_index,
                token_owner,
                CEP78Error::UnregisteredOwnerInBurn,
                false
            );
        }
    }

    fn update_page(
        &mut self,
        token_index: u64,
        token_owner: &Address,
        missing_table_error: CEP78Error,
        value: bool
    ) {
        // Get the page table, page address and page dictionary.
        let (page_table_entry, page_address, page_dict) = self.page_details(token_index);
        let mut page_table = self.get_page_table(token_owner, missing_table_error);

        // Check if the page is already allocated.
        let page_allocated = page_table[page_table_entry as usize];

        // If the page is not allocated, allocate a new page in the page table.
        if !page_allocated {
            page_table[page_table_entry as usize] = true;
            self.set_page_table(token_owner, page_table);
        }

        // Load page.
        let mut page = self.page(page_allocated, &page_dict, token_owner);

        // Update page value.
        page[page_address as usize] = value;
        self.set_page(&page_dict, token_owner, page);
    }

    // Based on the `allocated` flag, it returns either a new page or an empty page.
    pub fn page(&self, allocated: bool, page_dict: &str, token_owner: &Address) -> Vec<bool> {
        if !allocated {
            return vec![false; PAGE_SIZE as usize];
        }
        let item_key = utils::address_to_key(token_owner);
        self.env()
            .get_dictionary_value(page_dict, item_key.as_bytes())
            .unwrap_or_revert_with(self, CEP78Error::MissingPage)
    }

    // It returns:
    // - page_table_entry: the entry in the page table that maps to the underlying page
    // - page_address: the address in the page that maps to the token
    // - page_dict: the dictionary that holds the page
    pub fn page_details(&self, token_index: u64) -> (u64, u64, String) {
        let page_table_entry = token_index / PAGE_SIZE;
        let page_address = token_index % PAGE_SIZE;
        let page_dict = format!("{PREFIX_PAGE_DICTIONARY}_{}", page_table_entry);
        (page_table_entry, page_address, page_dict)
    }

    pub fn get_account_page_table(&self, owner: &Address) -> Option<Vec<bool>> {
        let owner_key = utils::address_to_key(owner);
        self.page_table.get(&owner_key)
    }

    pub fn get_token_index_checked(&self, token_identifier: &TokenIdentifier) -> u64 {
        match token_identifier {
            TokenIdentifier::Index(token_index) => *token_index,
            TokenIdentifier::Hash(_) => self
                .index_by_hash
                .get(&token_identifier.to_string())
                .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidTokenIdentifier)
        }
    }

    fn set_page(&mut self, page_dict: &str, token_owner: &Address, page: Vec<bool>) {
        let item_key = utils::address_to_key(token_owner);
        self.env()
            .set_dictionary_value(page_dict, item_key.as_bytes(), page);
    }

    // It returns the token index based on the token identifier.
    fn prepare_token_index(&mut self, token_identifier: &TokenIdentifier) -> u64 {
        match token_identifier {
            TokenIdentifier::Index(token_index) => *token_index,
            TokenIdentifier::Hash(token_hash) => {
                let token_index = self.index_by_hash.get(token_hash);

                // If the token exists, return the token index.
                if let Some(token_index) = token_index {
                    return token_index;
                }

                // If the token does not exist, create a new token index.
                let token_index = self.tokens_count.next_value();

                // Insert the token index into the hash table.
                self.index_by_hash.set(token_hash, token_index);
                self.hash_by_index.set(&token_index, token_hash.clone());

                // Return new token index.
                token_index
            }
        }
    }

    #[inline]
    fn get_mode(&self) -> OwnerReverseLookupMode {
        self.mode.get()
    }

    pub fn get_page_table(&self, owner: &Address, error: CEP78Error) -> Vec<bool> {
        let owner_key = utils::address_to_key(owner);
        self.page_table
            .get(&owner_key)
            .unwrap_or_revert_with(&self.env(), error)
    }

    pub fn set_page_table(&mut self, owner: &Address, page_table: Vec<bool>) {
        let owner_key = utils::address_to_key(owner);
        self.page_table.set(&owner_key, page_table);
    }
}
