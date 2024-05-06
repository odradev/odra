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
                use odra::UnwrapOrRevert;
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
macro_rules! compound_key_value_storage {
    ($name:ident, $dict:expr, $k1_type:ty, $k2_type:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key1: &$k1_type, key2: &$k2_type, value: $value_type) {
                use odra::UnwrapOrRevert;

                let env = self.env();
                let parts = [
                    key1.to_bytes().unwrap_or_revert(&env),
                    key2.to_bytes().unwrap_or_revert(&env)
                ];
                let key = crate::storage::compound_key(&env, &parts);
                env.set_dictionary_value($dict, &key, value);
            }

            pub fn get_or_default(&self, key1: &$k1_type, key2: &$k2_type) -> $value_type {
                use odra::UnwrapOrRevert;

                let env = self.env();
                let parts = [
                    key1.to_bytes().unwrap_or_revert(&env),
                    key2.to_bytes().unwrap_or_revert(&env)
                ];
                let key = crate::storage::compound_key(&env, &parts);
                env.get_dictionary_value($dict, &key).unwrap_or_default()
            }
        }
    };
    ($name:ident, $dict:expr, $k1_type:ty, $value_type:ty) => {
        compound_key_value_storage!($name, $dict, $k1_type, $k1_type, $value_type);
    };
}

#[macro_export]
macro_rules! encoded_key_value_storage {
    ($name:ident, $dict:expr, $key:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key: &$key, value: $value_type) {
                let env = self.env();
                let encoded_key = Self::key(&env, key);
                env.set_dictionary_value($dict, encoded_key.as_bytes(), value);
            }

            pub fn get(&self, key: &$key) -> Option<$value_type> {
                let env = self.env();
                let encoded_key = Self::key(&env, key);
                env.get_dictionary_value($dict, encoded_key.as_bytes())
            }

            fn key(env: &odra::ContractEnv, key: &$key) -> String {
                use base64::prelude::{Engine, BASE64_STANDARD};
                use odra::UnwrapOrRevert;

                let preimage = key.to_bytes().unwrap_or_revert(&env);
                BASE64_STANDARD.encode(preimage)
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
                self.env()
                    .set_dictionary_value($dict, key.as_bytes(), value);
            }

            pub fn get(&self, key: &str) -> Option<$value_type> {
                self.env().get_dictionary_value($dict, key.as_bytes())
            }
        }
    };
}

pub(crate) fn compound_key(env: &odra::ContractEnv, parts: &[odra::prelude::Vec<u8>]) -> [u8; 64] {
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
