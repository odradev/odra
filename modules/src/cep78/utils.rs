use odra::{
    casper_types::bytesrepr::FromBytes,
    Address, ContractEnv, OdraError, UnwrapOrRevert, Var
};

pub trait GetAs<T> {
    fn get_as(&self, env: &ContractEnv) -> T;
}

impl<R, T> GetAs<T> for Var<R>
where
    R: TryInto<T> + Default + FromBytes,
    R::Error: Into<OdraError>
{
    fn get_as(&self, env: &ContractEnv) -> T {
        self.get_or_default().try_into().unwrap_or_revert(env)
    }
}

pub trait IntoOrRevert<T> {
    type Error;
    fn into_or_revert(self, env: &ContractEnv) -> T;
}

impl<R, T> IntoOrRevert<T> for R
where
    R: TryInto<T>,
    R::Error: Into<OdraError>
{
    type Error = R::Error;
    fn into_or_revert(self, env: &ContractEnv) -> T {
        self.try_into().unwrap_or_revert(env)
    }
}

pub fn get_transfer_filter_contract() -> Option<Address> {
    None
}

// pub fn migrate_owned_tokens_in_ordinal_mode() {
//     let current_number_of_minted_tokens = utils::get_stored_value_with_user_errors::<u64>(
//         NUMBER_OF_MINTED_TOKENS,
//         NFTCoreError::MissingTotalTokenSupply,
//         NFTCoreError::InvalidTotalTokenSupply
//     );
//     let page_table_uref = get_uref(
//         PAGE_TABLE,
//         NFTCoreError::MissingPageTableURef,
//         NFTCoreError::InvalidPageTableURef
//     );
//     let page_table_width = get_stored_value_with_user_errors::<u64>(
//         PAGE_LIMIT,
//         NFTCoreError::MissingPageLimit,
//         NFTCoreError::InvalidPageLimit
//     );
//     let mut searched_token_ids: Vec<u64> = vec![];
//     for token_id in 0..current_number_of_minted_tokens {
//         if !searched_token_ids.contains(&token_id) {
//             let token_identifier = TokenIdentifier::new_index(token_id);
//             let token_owner_key = get_dictionary_value_from_key::<Key>(
//                 TOKEN_OWNERS,
//                 &token_identifier.get_dictionary_item_key()
//             )
//             .unwrap_or_revert_with(NFTCoreError::MissingNftKind);
//             let token_owner_item_key = encode_dictionary_item_key(token_owner_key);
//             let owned_tokens_list = get_token_identifiers_from_dictionary(
//                 &NFTIdentifierMode::Ordinal,
//                 &token_owner_item_key
//             )
//             .unwrap_or_revert();
//             for token_identifier in owned_tokens_list.into_iter() {
//                 let token_id = token_identifier.get_index().unwrap_or_revert();
//                 let page_number = token_id / PAGE_SIZE;
//                 let page_index = token_id % PAGE_SIZE;
//                 let mut page_record = match storage::dictionary_get::<Vec<bool>>(
//                     page_table_uref,
//                     &token_owner_item_key
//                 )
//                 .unwrap_or_revert()
//                 {
//                     Some(page_record) => page_record,
//                     None => vec![false; page_table_width as usize]
//                 };
//                 let page_uref = get_uref(
//                     &format!("{PREFIX_PAGE_DICTIONARY}_{page_number}"),
//                     NFTCoreError::MissingStorageUref,
//                     NFTCoreError::InvalidStorageUref
//                 );
//                 let _ = core::mem::replace(&mut page_record[page_number as usize], true);
//                 storage::dictionary_put(page_table_uref, &token_owner_item_key, page_record);
//                 let mut page =
//                     match storage::dictionary_get::<Vec<bool>>(page_uref, &token_owner_item_key)
//                         .unwrap_or_revert()
//                     {
//                         None => vec![false; PAGE_SIZE as usize],
//                         Some(single_page) => single_page
//                     };
//                 let is_already_marked_as_owned =
//                     core::mem::replace(&mut page[page_index as usize], true);
//                 if is_already_marked_as_owned {
//                     runtime::revert(NFTCoreError::InvalidPageIndex)
//                 }
//                 storage::dictionary_put(page_uref, &token_owner_item_key, page);
//                 searched_token_ids.push(token_id)
//             }
//         }
//     }
// }

// pub fn should_migrate_token_hashes(token_owner: Address) -> bool {
//     if get_token_identifiers_from_dictionary(
//         &NFTIdentifierMode::Hash,
//         &encode_dictionary_item_key(token_owner),
//     )
//     .is_none()
//     {
//         return false;
//     }
//     let page_table_uref = get_uref(
//         PAGE_TABLE,
//         NFTCoreError::MissingPageTableURef,
//         NFTCoreError::InvalidPageTableURef,
//     );
//     // If the owner has registered, then they will have an page table entry
//     // but it will contain no bits set.
//     let page_table = storage::dictionary_get::<Vec<bool>>(
//         page_table_uref,
//         &encode_dictionary_item_key(token_owner),
//     )
//     .unwrap_or_revert()
//     .unwrap_or_revert_with(NFTCoreError::UnregisteredOwnerFromMigration);
//     if page_table.contains(&true) {
//         return false;
//     }
//     true
// }

// pub fn migrate_token_hashes(token_owner: Key) {
//     let mut unmatched_hash_count = get_stored_value_with_user_errors::<u64>(
//         UNMATCHED_HASH_COUNT,
//         NFTCoreError::MissingUnmatchedHashCount,
//         NFTCoreError::InvalidUnmatchedHashCount
//     );

//     if unmatched_hash_count == 0 {
//         runtime::revert(NFTCoreError::InvalidNumberOfMintedTokens)
//     }

//     let token_owner_item_key = encode_dictionary_item_key(token_owner);
//     let owned_tokens_list =
//         get_token_identifiers_from_dictionary(&NFTIdentifierMode::Hash, &token_owner_item_key)
//             .unwrap_or_revert_with(NFTCoreError::InvalidTokenOwner);

//     let page_table_uref = get_uref(
//         PAGE_TABLE,
//         NFTCoreError::MissingPageTableURef,
//         NFTCoreError::InvalidPageTableURef
//     );

//     let page_table_width = get_stored_value_with_user_errors::<u64>(
//         PAGE_LIMIT,
//         NFTCoreError::MissingPageLimit,
//         NFTCoreError::InvalidPageLimit
//     );

//     for token_identifier in owned_tokens_list.into_iter() {
//         let token_address = unmatched_hash_count - 1;
//         let page_table_entry = token_address / PAGE_SIZE;
//         let page_address = token_address % PAGE_SIZE;
//         let mut page_table =
//             match storage::dictionary_get::<Vec<bool>>(page_table_uref, &token_owner_item_key)
//                 .unwrap_or_revert()
//             {
//                 Some(page_record) => page_record,
//                 None => vec![false; page_table_width as usize]
//             };
//         let _ = core::mem::replace(&mut page_table[page_table_entry as usize], true);
//         storage::dictionary_put(page_table_uref, &token_owner_item_key, page_table);
//         let page_uref = get_uref(
//             &format!("{PREFIX_PAGE_DICTIONARY}_{page_table_entry}"),
//             NFTCoreError::MissingStorageUref,
//             NFTCoreError::InvalidStorageUref
//         );
//         let mut page = match storage::dictionary_get::<Vec<bool>>(page_uref, &token_owner_item_key)
//             .unwrap_or_revert()
//         {
//             Some(single_page) => single_page,
//             None => vec![false; PAGE_SIZE as usize]
//         };
//         let _ = core::mem::replace(&mut page[page_address as usize], true);
//         storage::dictionary_put(page_uref, &token_owner_item_key, page);
//         insert_hash_id_lookups(unmatched_hash_count - 1, token_identifier);
//         unmatched_hash_count -= 1;
//     }

//     let unmatched_hash_count_uref = get_uref(
//         UNMATCHED_HASH_COUNT,
//         NFTCoreError::MissingUnmatchedHashCount,
//         NFTCoreError::InvalidUnmatchedHashCount
//     );

//     storage::write(unmatched_hash_count_uref, unmatched_hash_count);
// }
