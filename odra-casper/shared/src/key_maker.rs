use alloc::string::String;
use alloc::vec::Vec;
use odra_casper_types::casper_types::bytesrepr::Error;
use odra_casper_types::casper_types::bytesrepr::ToBytes;

// TODO: Add caching.
// TODO: Experiment with keys length.

/// Generate keys for storage.
pub trait KeyMaker {
    /// Generate key for variable.
    fn to_variable_key(key: &[u8]) -> Result<String, Error> {
        let hash = Self::blake2b(key);
        Ok(hex::encode(hash))
    }

    /// Generate key for dictionary.
    fn to_dictionary_key<T: ToBytes>(seed: &[u8], key: &T) -> Result<String, Error> {
        // TODO: Chagne to to_bytes when used in backend.
        let key_bytes = key.to_bytes()?;

        let mut preimage = Vec::with_capacity(seed.len() + key_bytes.len());
        preimage.extend_from_slice(seed);
        preimage.extend_from_slice(&key_bytes);

        let hash = Self::blake2b(&preimage);
        Ok(hex::encode(hash))
    }

    /// Hash value.
    fn blake2b(preimage: &[u8]) -> [u8; 32];
}
