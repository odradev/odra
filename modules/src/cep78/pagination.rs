use odra::{
    casper_types::{AccessRights, URef},
    prelude::*,
    Address, Mapping, UnwrapOrRevert
};

use crate::cep78::constants::PREFIX_PAGE_DICTIONARY;

use super::error::CEP78Error;

// The size of a given page, it is currently set to 1000
// to ease the math around addressing newly minted tokens.
pub const PAGE_SIZE: u64 = 1000;

#[odra::module]
pub struct Pagination {
    page_tables: Mapping<Address, Vec<bool>>,
    pages: Mapping<(String, u64, Address), Vec<bool>>
}

impl Pagination {
    pub fn add_page_entry_and_page_record(
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

        let mut page_table = match self.page_tables.get(item_key) {
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

    pub fn update_page_entry_and_page_record(
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
            .page_tables
            .get(&new_item_key)
            .unwrap_or_revert_with(&self.env(), CEP78Error::UnregisteredOwnerInTransfer);

        let mut target_page = if !target_page_table[page_table_entry as usize] {
            // Create a new page here
            let _ = core::mem::replace(&mut target_page_table[page_table_entry as usize], true);
            self.page_tables.set(new_item_key, target_page_table);
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

    pub fn register_owner(&mut self, owner: &Address) {
        if self.page_tables.get(&owner).is_none() {
            self.page_tables.set(owner, vec![false; PAGE_SIZE as usize]);
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
}
