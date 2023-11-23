use crate::prelude::*;
use casper_types::bytesrepr::{Error, ToBytes};

// TODO: don't know why clippy complains about this
#[allow(dead_code)]
const KEY_LENGTH: usize = 64;
static TABLE: &[u8] = b"0123456789abcdef";

#[inline]
fn hex(byte: u8) -> u8 {
    TABLE[byte as usize]
}

pub fn u32_to_hex(value: u32) -> [u8; 8] {
    let mut result = [0u8; 8];
    let bytes = value.to_be_bytes();
    for i in 0..4 {
        result[2 * i] = hex(bytes[i] >> 4);
        result[2 * i + 1] = hex(bytes[i] & 0xf);
    }
    result
}

pub fn bytes_to_hex(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    for byte in bytes {
        result.push(hex(byte >> 4));
        result.push(hex(byte & 0xf));
    }
    result
}

/// Generate keys for storage.
pub trait KeyMaker {
    /// Generate key for variable.
    fn to_variable_key(key: &[u8]) -> [u8; KEY_LENGTH] {
        Self::adjust_key(key)
    }

    /// Generate key for dictionary.
    fn to_dictionary_key<'a, T: ToBytes>(
        seed: &'a [u8],
        key: &'a T
    ) -> Result<[u8; KEY_LENGTH], Error> {
        let key_hash = key.to_bytes()?;

        let storage_key_len = seed.len() + key_hash.len();

        let mut storage_key = Vec::with_capacity(storage_key_len);
        storage_key.extend_from_slice(seed);
        storage_key.extend_from_slice(&key_hash);

        Ok(Self::adjust_key(&storage_key))
    }

    #[inline]
    fn adjust_key(preimage: &[u8]) -> [u8; KEY_LENGTH] {
        let hash: [u8; 32] = Self::blake2b(preimage);
        let mut result = [0; KEY_LENGTH];
        crate::utils::hex_to_slice(&hash, &mut result);
        result
    }

    /// Hash value.
    fn blake2b(preimage: &[u8]) -> [u8; 32];
}

#[cfg(test)]
mod tests {
    use crate::key_maker::u32_to_hex;

    #[test]
    fn test_u32_to_hex() {
        assert_eq!(&u32_to_hex(0), b"00000000");
        assert_eq!(&u32_to_hex(255), b"000000ff");
    }
}
