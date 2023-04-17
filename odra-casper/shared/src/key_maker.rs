use odra_casper_types::casper_types::bytesrepr::Error;
use odra_casper_types::casper_types::bytesrepr::ToBytes;

// TODO: Add caching.
// TODO: Experiment with keys length.
pub trait KeyMaker {
    fn to_variable_key(key: &str) -> Result<String, Error> {
        let preimage = key.to_bytes()?;
        let hash = Self::blake2b(&preimage);
        Ok(hex::encode(hash))
    }

    fn to_dictionary_key<T: ToBytes>(seed: &str, key: &T) -> Result<String, Error> {
        // TODO: Chagne to to_bytes when used in backend.
        let seed_bytes = seed.as_bytes();
        let key_bytes = key.to_bytes()?;

        let mut preimage = Vec::with_capacity(seed_bytes.len() + key_bytes.len());
        preimage.extend_from_slice(seed_bytes);
        preimage.extend_from_slice(&key_bytes);

        let hash = Self::blake2b(&preimage);
        Ok(hex::encode(hash))
    }

    fn blake2b(preimage: &[u8]) -> [u8; 32];
}
