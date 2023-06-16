use alloc::vec::Vec;
use casper_event_standard::casper_types::bytesrepr::Error;
use odra_casper_types::OdraType;

pub const KEY_LENGTH: usize = 64;

/// Generate keys for storage.
pub trait KeyMaker {
    /// Generate key for variable.
    fn to_variable_key(key: &[u8], dest: &mut [u8]) {
        Self::adjust_key(key, dest)
    }

    /// Generate key for dictionary.
    fn to_dictionary_key<'a, T: OdraType>(
        seed: &'a [u8],
        key: &'a T,
        dest: &mut [u8]
    ) -> Result<(), Error> {
        let key_hash = key.to_bytes()?;

        let storage_key_len = seed.len() + key_hash.len();

        let mut storage_key = Vec::with_capacity(storage_key_len);
        storage_key.extend_from_slice(seed);
        storage_key.extend_from_slice(&key_hash);

        Self::adjust_key(&storage_key, dest);
        Ok(())
    }

    #[inline]
    fn adjust_key(preimage: &[u8], dest: &mut [u8]) {
        let hash: [u8; 32] = Self::blake2b(preimage);
        odra_utils::hex_to_slice(&hash, dest);
    }

    /// Hash value.
    fn blake2b(preimage: &[u8]) -> [u8; 32];
}
