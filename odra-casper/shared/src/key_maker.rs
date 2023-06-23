use alloc::vec::Vec;

pub enum StorageKey<'a> {
    Ref(&'a [u8]),
    Owned(Vec<u8>)
}

impl<'a> AsRef<[u8]> for StorageKey<'a> {
    fn as_ref(&self) -> &[u8] {
        match self {
            StorageKey::Ref(bytes) => bytes,
            StorageKey::Owned(vec) => vec.as_slice()
        }
    }
}

/// Generate keys for storage.
pub trait KeyMaker {
    const DICTIONARY_ITEM_KEY_MAX_LENGTH: usize;

    /// Generate key for variable.
    fn to_variable_key(key: &[u8]) -> StorageKey {
        if key.len() > Self::DICTIONARY_ITEM_KEY_MAX_LENGTH {
            StorageKey::Owned(Self::adjust_key(key))
        } else {
            StorageKey::Ref(key)
        }
    }

    /// Generate key for dictionary.
    fn to_dictionary_key<'a, T: odra_casper_types::Key>(
        seed: &'a [u8],
        key: &'a T
    ) -> StorageKey<'a> {
        let key_hash = key.encoded_hash();

        let storage_key_len = seed.len() + key_hash.len();

        let mut storage_key = Vec::with_capacity(storage_key_len);
        storage_key.extend_from_slice(seed);
        storage_key.extend_from_slice(&key_hash);

        if storage_key_len > Self::DICTIONARY_ITEM_KEY_MAX_LENGTH {
            StorageKey::Owned(Self::adjust_key(&storage_key))
        } else {
            StorageKey::Owned(storage_key)
        }
    }

    fn adjust_key(preimage: &[u8]) -> Vec<u8> {
        let hash = Self::blake2b(preimage);
        let mut result  = Vec::with_capacity(64);
        odra_utils::hex_to_slice(&hash, &mut result);
        result
    }

    /// Hash value.
    fn blake2b(preimage: &[u8]) -> [u8; 32];
}
