// use casper_types::URef;
// use odra_core::prelude::Address;

// /// Gets an uref for a main purse of an account or a contract.
// pub async fn get_main_purse(&self, address: &Address) -> Result<URef, String> {
//     let maybe_purse_uref = self.query_global_state_maybe(address.as_key(), None).await;
//     let purse_uref_value = match maybe_purse_uref {
//         None => {
//             return Err(format!(
//                 "Couldn't get purse uref for address: {:?}",
//                 address.to_formatted_string()
//             ));
//         }
//         Some(p) => p
//     };

//     match purse_uref_value {
//         CLValue(value) => value.into_t().unwrap(),
//         StoredValue::AddressableEntity(entity) => entity.main_purse(),
//         StoredValue::Account(account) => account.main_purse(),
//         StoredValue::ContractPackage(contract_package) => {
//             let last_version = contract_package.current_contract_hash().unwrap();
//             let maybe_contract = self
//                 .query_global_state_maybe(Key::Hash(last_version.value()), None)
//                 .await;
//             let contract_value = match maybe_contract {
//                 None => {
//                     panic!(
//                         "Couldn't get contract for address: {:?}",
//                         address.to_formatted_string()
//                     )
//                 }
//                 Some(c) => c
//             };
//             match contract_value {
//                 StoredValue::Contract(contract) => contract
//                     .named_keys()
//                     .get(CONTRACT_MAIN_PURSE)
//                     .unwrap()
//                     .into_uref()
//                     .unwrap(),
//                 _ => panic!(
//                     "Couldn't get main purse for address: {:?}",
//                     address.to_formatted_string()
//                 )
//             }
//         }
//         _ => panic!(
//             "Getting main purse is not supported for: {:?}",
//             purse_uref_value
//         )
//     }
// }
