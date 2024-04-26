pub const ACL_PACKAGE_MODE: &str = "acl_package_mode";
pub const ACL_WHITELIST: &str = "acl_whitelist";
pub const ALLOW_MINTING: &str = "allow_minting";
pub const APPROVED: &str = "approved";
pub const BURN_MODE: &str = "burn_mode";
pub const BURNT_TOKENS: &str = "burnt_tokens";
pub const COLLECTION_NAME: &str = "collection_name";
pub const COLLECTION_SYMBOL: &str = "collection_symbol";
pub const CONTRACT_WHITELIST: &str = "contract_whitelist";
pub const EVENT_TYPE: &str = "event_type";
pub const EVENTS: &str = "events";
pub const EVENTS_MODE: &str = "events_mode";
pub const HASH_BY_INDEX: &str = "hash_by_index";
pub const HOLDER_MODE: &str = "holder_mode";
pub const IDENTIFIER_MODE: &str = "identifier_mode";
pub const INDEX_BY_HASH: &str = "index_by_hash";
pub const INSTALLER: &str = "installer";
pub const JSON_SCHEMA: &str = "json_schema";
pub const METADATA_CEP78: &str = "metadata_cep78";
pub const METADATA_CUSTOM_VALIDATED: &str = "metadata_custom_validated";
pub const METADATA_MUTABILITY: &str = "metadata_mutability";
pub const METADATA_NFT721: &str = "metadata_nft721";
pub const METADATA_RAW: &str = "metadata_raw";
pub const MIGRATION_FLAG: &str = "migration_flag";
pub const MINTING_MODE: &str = "minting_mode";
pub const NFT_KIND: &str = "nft_kind";
pub const NFT_METADATA_KIND: &str = "nft_metadata_kind";
pub const NFT_METADATA_KINDS: &str = "nft_metadata_kinds";
pub const NUMBER_OF_MINTED_TOKENS: &str = "number_of_minted_tokens";
pub const OPERATOR: &str = "operator";
pub const OPERATORS: &str = "operators";
pub const OPERATOR_BURN_MODE: &str = "operator_burn_mode";
pub const OWNED_TOKENS: &str = "owned_tokens";
pub const OWNER: &str = "owner";
pub const BURNER: &str = "burner";
pub const OWNERSHIP_MODE: &str = "ownership_mode";
pub const PACKAGE_OPERATOR_MODE: &str = "package_operator_mode";
pub const PAGE_LIMIT: &str = "page_limit";
pub const PAGE_TABLE: &str = "page_table";
pub const RECEIPT_NAME: &str = "receipt_name";
pub const RECIPIENT: &str = "recipient";
pub const REPORTING_MODE: &str = "reporting_mode";
pub const RLO_MFLAG: &str = "rlo_mflag";
pub const SENDER: &str = "sender";
pub const SPENDER: &str = "spender";
pub const TOKEN_COUNT: &str = "balances";
pub const TOKEN_ID: &str = "token_id";
pub const TOKEN_ISSUERS: &str = "token_issuers";
pub const TOKEN_OWNERS: &str = "token_owners";
pub const TOTAL_TOKEN_SUPPLY: &str = "total_token_supply";
pub const TRANSFER_FILTER_CONTRACT: &str = "transfer_filter_contract";
pub const TRANSFER_FILTER_CONTRACT_METHOD: &str = "can_transfer";
pub const UNMATCHED_HASH_COUNT: &str = "unmatched_hash_count";
pub const WHITELIST_MODE: &str = "whitelist_mode";

#[macro_export]
macro_rules! simple_storage {
    ($name:ident, $value_ty:ty, $key:ident, $err:expr) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, value: $value_ty) {
                self.env().set_named_value($key, value);
            }

            pub fn get(&self) -> $value_ty {
                self.env()
                    .get_named_value($key)
                    .unwrap_or_revert_with(&self.env(), $err)
            }
        }
    };
    ($name:ident, $value_ty:ty, $key:ident) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, value: $value_ty) {
                self.env().set_named_value($key, value);
            }

            pub fn get(&self) -> Option<$value_ty> {
                self.env().get_named_value($key)
            }
        }
    };
}

#[macro_export]
macro_rules! compound_key_storage {
    ($name:ident, $dict:expr, $k1_type:ty, $k2_type:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key1: &$k1_type, key2: &$k2_type, value: $value_type) {
                let env = self.env();
                let parts = [
                    key1.to_bytes().unwrap_or_revert(&env),
                    key2.to_bytes().unwrap_or_revert(&env),
                ];
                let key = crate::cep78::storage::compound_key(&env, &parts);
                env.set_dictionary_value($dict, &key, value);
            }

            pub fn get_or_default(&self, key1: &$k1_type, key2: &$k2_type) -> $value_type {
                let env = self.env();
                let parts = [
                    key1.to_bytes().unwrap_or_revert(&env),
                    key2.to_bytes().unwrap_or_revert(&env),
                ];
                let key = crate::cep78::storage::compound_key(&env, &parts);
                env.get_dictionary_value($dict, &key).unwrap_or_default()
            }
        }
    };
    ($name:ident, $dict:expr, $k1_type:ty, $value_type:ty) => {
        compound_key_storage!(
            $name,
            $dict,
            $k1_type,
            $k1_type,
            $value_type
        );
    }
}

#[macro_export]
macro_rules! encoded_key_value_storage {
    ($name:ident, $dict:expr, $key:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key: &$key, value: $value_type) {
                let env = self.env();
                let preimage = key.to_bytes().unwrap_or_revert(&env);
                let key = BASE64_STANDARD.encode(preimage);
                env.set_dictionary_value($dict, key.as_bytes(), value);
            }

            pub fn get(&self, key: &$key) -> Option<$value_type> {
                let env = self.env();
                let preimage = key.to_bytes().unwrap_or_revert(&env);
                let key = BASE64_STANDARD.encode(preimage);
                env.get_dictionary_value($dict, key.as_bytes())
            }
        }
    };
}

#[macro_export]
macro_rules! basic_key_value_storage {
    ($name:ident, $dict:expr, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key: &str, value: $value_type) {
                self.env().set_dictionary_value($dict, key.as_bytes(), value);
            }

            pub fn get(&self, key: &str) -> Option<$value_type> {
                self.env().get_dictionary_value($dict, key.as_bytes())
            }
        }
    };
}

pub fn compound_key(env: &odra::ContractEnv, parts: &[odra::prelude::Vec<u8>]) -> [u8; 64] {
    use odra::casper_types::bytesrepr::ToBytes;
    use odra::UnwrapOrRevert;

    let mut result = [0u8; 64];
    let mut preimage = odra::prelude::Vec::new();
    for part in parts {
        preimage.append(&mut part.to_bytes().unwrap_or_revert(env));
    }

    let key_bytes = env.hash(&preimage);
    odra::utils::hex_to_slice(&key_bytes, &mut result);
    result
}