use casper_types::{Key, StoredValue, URef};
use odra_core::prelude::Address;

use crate::OdraWasmClient;

impl OdraWasmClient {
    /// Gets an uref for a main purse of an account or a contract.
    pub async fn get_main_purse(&self, address: &Address) -> Result<URef, String> {
        let purse_uref_value = self
            .query_global_state(address.as_key(), None)
            .await
            .ok_or(format!(
                "Couldn't get purse uref for address: {:?}",
                address.to_formatted_string()
            ))?;

        let result = match purse_uref_value {
            StoredValue::CLValue(value) => value.into_t().map_err(|_| format!(
                "Couldn't get CLValue for address: {:?}",
                address.to_formatted_string()
            )),
            StoredValue::AddressableEntity(entity) => Ok(entity.main_purse()),
            StoredValue::Account(account) => Ok(account.main_purse()),
            StoredValue::ContractPackage(contract_package) => {
                let last_version = contract_package.current_contract_hash().ok_or(format!(
                    "Couldn't get last version for address: {:?}",
                    address.to_formatted_string()
                ))?;
                let maybe_contract = self
                    .query_global_state(Key::Hash(last_version.value()), None)
                    .await;
                match maybe_contract {
                    Some(StoredValue::Contract(contract)) => contract
                        .named_keys()
                        .get("__contract_main_purse")
                        .map(|v| v.into_uref())
                        .flatten()
                        .ok_or(format!(
                            "Couldn't get main purse for address: {:?}",
                            address.to_formatted_string()
                        )),
                    _ => Err(format!(
                        "Couldn't get main purse for address: {:?}",
                        address.to_formatted_string()
                    ))
                }
            }
            _ => {
                return Err(format!(
                    "Getting main purse is not supported for: {:?}",
                    purse_uref_value
                ))
            }
        };
        result
    }
}
