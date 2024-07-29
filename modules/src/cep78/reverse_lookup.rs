use odra::{
    args::Maybe,
    casper_types::{AccessRights, URef},
    named_keys::{key_value_storage, single_value_storage},
    prelude::*,
    Address, Mapping, SubModule, UnwrapOrRevert
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
    page_limit: SubModule<Cep78PageLimit>
}

impl ReverseLookup {
    pub fn init(&mut self, mode: OwnerReverseLookupMode, receipt_name: String) {
        self.mode.set(mode);
        self.receipt_name.set(receipt_name);

        if [
            OwnerReverseLookupMode::Complete,
            OwnerReverseLookupMode::TransfersOnly
        ]
        .contains(&mode)
        {
            let page_table_width = utils::max_number_of_pages(0);
            self.page_limit.set(page_table_width);
        }
    }

    /// Insert a hash into the reverse lookup table.
    /// Returns:
    /// - true if the hash was inserted,
    /// - false if it was already present,
    pub fn insert_hash(
        &mut self,
        current_number_of_minted_tokens: u64,
        token_identifier: &TokenIdentifier
    ) -> bool {
        let token_identifier = token_identifier.get_hash().unwrap();

        let inserted_token_index = self
            .index_by_hash
            .get(&token_identifier);

        // If data is not inserted, then insert it.
        if inserted_token_index.is_none() {
            self.index_by_hash.set(
                &token_identifier,
                current_number_of_minted_tokens
            );
            self.hash_by_index.set(
                &current_number_of_minted_tokens,
                token_identifier
            );
            return true;
        }

        // If token identifier points at the index.
        // Check if the index points back to the token identifier.
        if let Some(token_index) = inserted_token_index.clone() {
            let token_hash = self
                .hash_by_index
                .get(&token_index);
            if let Some(token_hash) = token_hash {
                if token_hash == token_identifier {
                    return false;
                }
            }
        }

        // In other case the data is not inserted correctly, and should be reverted.
        self.env().revert(CEP78Error::ReverseLookupIntegrityViolation);
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

    pub fn on_mint(&mut self, tokens_count: u64, token_owner: Address, _token_id: String) {
        if self.get_mode() == OwnerReverseLookupMode::Complete {
            let token_owner_key = utils::address_to_key(&token_owner);
            let (_page_table_entry, _page_uref) =
                self.add_page_entry_and_page_record(tokens_count, &token_owner_key, true);
            // Uncomment if deciding to return the receipt

            // let receipt_name = self.receipt_name.get();
            // let receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // let receipt_address = Key::dictionary(page_uref, token_owner_key.as_bytes());
            // return (receipt_string, receipt_address, token_id);
        }
    }

    pub fn on_transfer(
        &mut self,
        token_identifier: TokenIdentifier,
        source: Address,
        target: Address
    ) {
        let mode = self.get_mode();
        if let OwnerReverseLookupMode::Complete | OwnerReverseLookupMode::TransfersOnly = mode {
            // Update to_account owned_tokens. Revert if owned_tokens list is not found
            let tokens_count = self.get_token_index(&token_identifier);
            let source_key = utils::address_to_key(&source);
            let target_key = utils::address_to_key(&target);
            if OwnerReverseLookupMode::TransfersOnly == mode {
                self.add_page_entry_and_page_record(tokens_count, &source_key, false);
            }

            let (_page_table_entry, _page_uref) =
                self.update_page_entry_and_page_record(tokens_count, &source_key, &target_key);
            // Uncomment if deciding to return the receipt

            // let receipt_name = self.receipt_name.get();
            // let receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // let owned_tokens_actual_key = Key::dictionary(page_uref, source_key.as_bytes());
            // return (receipt_string, owned_tokens_actual_key);
        }
    }

    fn add_page_entry_and_page_record(
        &mut self,
        tokens_count: u64,
        item_key: &str,
        on_mint: bool
    ) -> (u64, URef) {
        // there is an explicit page_table;
        // this is the entry in that overall page table which maps to the underlying page
        // upon which this mint's address will exist
        let env = self.env();
        let page_table_entry = tokens_count / PAGE_SIZE;
        let page_address = tokens_count % PAGE_SIZE;

        let mut page_table = match self.page_table.get(item_key) {
            Some(page_table) => page_table,
            None => env.revert(if on_mint {
                CEP78Error::UnregisteredOwnerInMint
            } else {
                CEP78Error::UnregisteredOwnerInTransfer
            })
        };

        let page_dict = format!("{PREFIX_PAGE_DICTIONARY}_{}", page_table_entry);

        let mut page = if !page_table[page_table_entry as usize] {
            // We mark the page table entry to true to signal the allocation of a page.
            let _ = core::mem::replace(&mut page_table[page_table_entry as usize], true);
            self.page_table.set(item_key, page_table);
            vec![false; PAGE_SIZE as usize]
        } else {
            env.get_dictionary_value(&page_dict, item_key.as_bytes())
                .unwrap_or_revert_with(&env, CEP78Error::MissingPage)
        };

        let _ = core::mem::replace(&mut page[page_address as usize], true);
        env.set_dictionary_value(page_dict, item_key.as_bytes(), page);

        // TODO: return the page_uref
        let addr_array = [0u8; 32];
        let uref_a = URef::new(addr_array, AccessRights::READ);
        (page_table_entry, uref_a)
    }

    fn update_page_entry_and_page_record(
        &mut self,
        tokens_count: u64,
        old_item_key: &str,
        new_item_key: &str
    ) -> (u64, URef) {
        let env = self.env();
        let page_table_entry = tokens_count / PAGE_SIZE;
        let page_address = tokens_count % PAGE_SIZE;
        let page_dict = format!("{PREFIX_PAGE_DICTIONARY}_{}", page_table_entry);

        let mut source_page: Vec<bool> = env
            .get_dictionary_value(&page_dict, old_item_key.as_bytes())
            .unwrap_or_revert_with(&env, CEP78Error::InvalidPageNumber);

        if !source_page[page_address as usize] {
            env.revert(CEP78Error::InvalidTokenIdentifier)
        }

        let _ = core::mem::replace(&mut source_page[page_address as usize], false);

        env.set_dictionary_value(&page_dict, old_item_key.as_bytes(), source_page);

        let mut target_page_table = self
            .page_table
            .get(new_item_key)
            .unwrap_or_revert_with(&env, CEP78Error::UnregisteredOwnerInTransfer);

        let mut target_page = if !target_page_table[page_table_entry as usize] {
            // Create a new page here
            let _ = core::mem::replace(&mut target_page_table[page_table_entry as usize], true);
            self.page_table.set(new_item_key, target_page_table);
            vec![false; PAGE_SIZE as usize]
        } else {
            env.get_dictionary_value(&page_dict, new_item_key.as_bytes())
                .unwrap_or_revert(self)
        };

        let _ = core::mem::replace(&mut target_page[page_address as usize], true);

        env.set_dictionary_value(&page_dict, new_item_key.as_bytes(), target_page);

        let addr_array = [0u8; 32];
        let uref_a = URef::new(addr_array, AccessRights::READ);
        // (page_table_entry, page_uref)
        (page_table_entry, uref_a)
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

    #[inline]
    fn get_mode(&self) -> OwnerReverseLookupMode {
        self.mode.get()
    }

    pub fn get_page_table(&self, owner: Address) -> Option<Vec<bool>> {
        let owner_key = utils::address_to_key(&owner);
        self.page_table.get(&owner_key)
    }
}
