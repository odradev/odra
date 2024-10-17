/// Creates an Odra module that stores a single value under a given named key.
/// The module has two methods: `set` and `get`.
/// If the value is not set and an error is passed as the fourth argument, `get` will revert with the provided error.
#[macro_export]
macro_rules! single_value_storage {
    ($name:ident, $value_ty:ty, $key:expr, $err:expr) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, value: $value_ty) {
                self.env().set_named_value($key, value);
            }

            pub fn get(&self) -> $value_ty {
                self.env()
                    .get_named_value($key)
                    .unwrap_or_revert_with(self, $err)
            }
        }
    };
    ($name:ident, $value_ty:ty, $key:expr) => {
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

/// Creates an Odra module that stores a values in a given dictionary.
/// The module has two methods: `set` and `get`.
/// The `key` argument of `set` and `get` is used as a dictionary key.
#[macro_export]
macro_rules! key_value_storage {
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

/// Creates an Odra module that stores a values in a given dictionary.
/// The module has two methods: `set` and `get`.
/// The `key` argument of `set` and `get` is base64-encoded and then used as a dictionary key.
#[macro_export]
macro_rules! base64_encoded_key_value_storage {
    ($name:ident, $dict:expr, $key:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key: &$key, value: $value_type) {
                let encoded_key = Self::key(self, key);
                self.env()
                    .set_dictionary_value($dict, encoded_key.as_bytes(), value);
            }

            pub fn get(&self, key: &$key) -> Option<$value_type> {
                let encoded_key = Self::key(self, key);
                self.env()
                    .get_dictionary_value($dict, encoded_key.as_bytes())
            }

            #[inline]
            fn key<R: odra::module::Revertible>(rev: &R, key: &$key) -> String {
                use base64::prelude::{Engine, BASE64_STANDARD};

                let preimage = key.to_bytes().unwrap_or_revert(rev);
                BASE64_STANDARD.encode(preimage)
            }
        }
    };
}

/// Creates an Odra module that stores a values in a given dictionary.
/// The module has two methods: `set` and `get`.
/// The `key1` and `key2` arguments of `set` and `get` are converted to bytes, combined into a single bytes vector,
/// and finally hex-encoded and then used as a dictionary key.
#[macro_export]
macro_rules! compound_key_value_storage {
    ($name:ident, $dict:expr, $k1_type:ty, $k2_type:ty, $value_type:ty) => {
        #[odra::module]
        pub struct $name;

        impl $name {
            pub fn set(&self, key1: &$k1_type, key2: &$k2_type, value: $value_type) {
                let mut key = [0u8; 64];
                let mut preimage = odra::prelude::Vec::new();
                preimage.extend_from_slice(&key1.to_bytes().unwrap_or_revert(self));
                preimage.extend_from_slice(&key2.to_bytes().unwrap_or_revert(self));

                let env = self.env();
                let key_bytes = env.hash(&preimage);
                odra::utils::hex_to_slice(&key_bytes, &mut key);
                env.set_dictionary_value($dict, &key, value);
            }

            pub fn get_or_default(&self, key1: &$k1_type, key2: &$k2_type) -> $value_type {
                let mut key = [0u8; 64];
                let mut preimage = odra::prelude::Vec::new();
                preimage.extend_from_slice(&key1.to_bytes().unwrap_or_revert(self));
                preimage.extend_from_slice(&key2.to_bytes().unwrap_or_revert(self));

                let env = self.env();
                let key_bytes = env.hash(&preimage);
                odra::utils::hex_to_slice(&key_bytes, &mut key);
                env.get_dictionary_value($dict, &key).unwrap_or_default()
            }
        }
    };
    ($name:ident, $dict:expr, $k1_type:ty, $value_type:ty) => {
        compound_key_value_storage!($name, $dict, $k1_type, $k1_type, $value_type);
    };
}
