use odra::{
    args::Maybe,
    casper_types::{AccessRights, URef},
    prelude::*,
    Address, Mapping, UnwrapOrRevert, Var
};

use super::{
    constants::PREFIX_PAGE_DICTIONARY,
    error::CEP78Error,
    modalities::{OwnerReverseLookupMode, OwnershipMode, TokenIdentifier}
};
// The size of a given page, it is currently set to 1000
// to ease the math around addressing newly minted tokens.
pub const PAGE_SIZE: u64 = 1000;

#[odra::module]
pub struct ReverseLookup {
    mode: Var<OwnerReverseLookupMode>,
    hash_by_index: Mapping<u64, String>,
    index_by_hash: Mapping<String, u64>,
    page_table: Mapping<Address, Vec<bool>>,
    pages: Mapping<(String, u64, Address), Vec<bool>>,
    receipt_name: Var<String>
}

impl ReverseLookup {
    pub fn init(&mut self, mode: OwnerReverseLookupMode, receipt_name: String) {
        self.mode.set(mode);
        self.receipt_name.set(receipt_name);
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

    pub fn register_owner(
        &mut self,
        owner: Maybe<Address>,
        ownership_mode: OwnershipMode
    ) -> (String, URef) {
        let mode = self.get_mode();
        if [
            OwnerReverseLookupMode::Complete,
            OwnerReverseLookupMode::TransfersOnly
        ]
        .contains(&mode)
        {
            let owner = match ownership_mode {
                OwnershipMode::Minter => self.env().caller(),
                OwnershipMode::Assigned | OwnershipMode::Transferable => owner.unwrap(&self.env())
            };
            if self.page_table.get(&owner).is_none() {
                self.page_table.set(&owner, vec![false; PAGE_SIZE as usize]);
            }

            // let page_table_uref = utils::get_uref(
            //     PAGE_TABLE,
            //     NFTCoreError::MissingPageTableURef,
            //     NFTCoreError::InvalidPageTableURef,
            // );

            // let owner_item_key = utils::encode_dictionary_item_key(owner_key);

            // if storage::dictionary_get::<Vec<bool>>(page_table_uref, &owner_item_key)
            //     .unwrap_or_revert()
            //     .is_none()
            // {
            //     let page_table_width = utils::get_stored_value_with_user_errors::<u64>(
            //         PAGE_LIMIT,
            //         NFTCoreError::MissingPageLimit,
            //         NFTCoreError::InvalidPageLimit,
            //     );
            //     storage::dictionary_put(
            //         page_table_uref,
            //         &owner_item_key,
            //         vec![false; page_table_width as usize],
            //     );
            // }
            // let collection_name = utils::get_stored_value_with_user_errors::<String>(
            //     COLLECTION_NAME,
            //     NFTCoreError::MissingCollectionName,
            //     NFTCoreError::InvalidCollectionName,
            // );
            // let package_uref = storage::new_uref(utils::get_stored_value_with_user_errors::<String>(
            //     &format!("{PREFIX_CEP78}_{collection_name}"),
            //     NFTCoreError::MissingCep78PackageHash,
            //     NFTCoreError::InvalidCep78InvalidHash,
            // ));
            // runtime::ret(CLValue::from_t((collection_name, package_uref)).unwrap_or_revert())
        }
        ("".to_string(), URef::new([0u8; 32], AccessRights::READ))
    }

    pub fn on_mint(
        &mut self,
        tokens_count: u64,
        token_owner: Address,
        token_id: String
    ) -> (String, Address, String) {
        if self.get_mode() == OwnerReverseLookupMode::Complete {
            let (page_table_entry, _page_uref) =
                self.add_page_entry_and_page_record(tokens_count, &token_owner, true);

            let receipt_name = self.receipt_name.get_or_default();
            let receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // TODO: Implement the following
            // let receipt_address = Key::dictionary(page_uref, owned_tokens_item.as_bytes());
            // should not return `token_owner`
            return (receipt_string, token_owner, token_id);
        }
        ("".to_string(), token_owner, token_id)
    }

    pub fn on_transfer(
        &mut self,
        token_identifier: TokenIdentifier,
        source: Address,
        _target: Address
    ) -> (String, Address) {
        let mode = self.get_mode();
        if let OwnerReverseLookupMode::Complete | OwnerReverseLookupMode::TransfersOnly = mode {
            // Update to_account owned_tokens. Revert if owned_tokens list is not found
            let tokens_count = self.get_token_index(&token_identifier);
            if OwnerReverseLookupMode::TransfersOnly == mode {
                self.add_page_entry_and_page_record(tokens_count, &source, false);
            }
            // TODO: Implement the following

            // let (page_table_entry, _page_uref) =
            //     self.update_page_entry_and_page_record(tokens_count, &source, &target);
            // let receipt_name = self.receipt_name.get_or_default();
            // let _receipt_string = format!("{receipt_name}_m_{PAGE_SIZE}_p_{page_table_entry}");
            // let receipt_address = Key::dictionary(page_uref, owned_tokens_item_key.as_bytes());
            // return (receipt_string, source);
        }
        ("".to_owned(), source)
    }

    fn add_page_entry_and_page_record(
        &mut self,
        tokens_count: u64,
        item_key: &Address,
        on_mint: bool
    ) -> (u64, URef) {
        // there is an explicit page_table;
        // this is the entry in that overall page table which maps to the underlying page
        // upon which this mint's address will exist
        let page_table_entry = tokens_count / PAGE_SIZE;
        let page_address = tokens_count % PAGE_SIZE;

        let mut page_table = match self.page_table.get(item_key) {
            Some(page_table) => page_table,
            None => self.env().revert(if on_mint {
                CEP78Error::UnregisteredOwnerInMint
            } else {
                CEP78Error::UnregisteredOwnerInTransfer
            })
        };

        let page_key = (
            PREFIX_PAGE_DICTIONARY.to_string(),
            page_table_entry,
            *item_key
        );
        let mut page = if !page_table[page_table_entry as usize] {
            // We mark the page table entry to true to signal the allocation of a page.
            let _ = core::mem::replace(&mut page_table[page_table_entry as usize], true);
            self.pages.set(&page_key, page_table);
            vec![false; PAGE_SIZE as usize]
        } else {
            self.pages
                .get(&page_key)
                .unwrap_or_revert_with(&self.env(), CEP78Error::MissingPage)
        };

        let _ = core::mem::replace(&mut page[page_address as usize], true);

        self.pages.set(&page_key, page);
        // storage::dictionary_put(page_uref, item_key, page);
        let addr_array = [0u8; 32];
        let uref_a = URef::new(addr_array, AccessRights::READ);
        // (page_table_entry, page_uref)
        (page_table_entry, uref_a)
    }

    fn _update_page_entry_and_page_record(
        &mut self,
        tokens_count: u64,
        old_item_key: &Address,
        new_item_key: &Address
    ) -> (u64, URef) {
        let page_table_entry = tokens_count / PAGE_SIZE;
        let page_address = tokens_count % PAGE_SIZE;

        let old_page_key = (
            PREFIX_PAGE_DICTIONARY.to_string(),
            page_table_entry,
            *old_item_key
        );
        let new_page_key = (
            PREFIX_PAGE_DICTIONARY.to_string(),
            page_table_entry,
            *new_item_key
        );

        let mut source_page = self
            .pages
            .get(&old_page_key)
            .unwrap_or_revert_with(&self.env(), CEP78Error::InvalidPageNumber);

        if !source_page[page_address as usize] {
            self.env().revert(CEP78Error::InvalidTokenIdentifier)
        }

        let _ = core::mem::replace(&mut source_page[page_address as usize], false);

        self.pages.set(&old_page_key, source_page);

        let mut target_page_table = self
            .page_table
            .get(new_item_key)
            .unwrap_or_revert_with(&self.env(), CEP78Error::UnregisteredOwnerInTransfer);

        let mut target_page = if !target_page_table[page_table_entry as usize] {
            // Create a new page here
            let _ = core::mem::replace(&mut target_page_table[page_table_entry as usize], true);
            self.page_table.set(new_item_key, target_page_table);
            vec![false; PAGE_SIZE as usize]
        } else {
            self.pages.get(&new_page_key).unwrap_or_revert(&self.env())
        };

        let _ = core::mem::replace(&mut target_page[page_address as usize], true);

        self.pages.set(&new_page_key, target_page);
        // (page_table_entry, page_uref)
        let addr_array = [0u8; 32];
        let uref_a = URef::new(addr_array, AccessRights::READ);
        // (page_table_entry, page_uref)
        (page_table_entry, uref_a)
    }

    fn get_token_index(&self, token_identifier: &TokenIdentifier) -> u64 {
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
        self.mode.get_or_default()
    }
}
