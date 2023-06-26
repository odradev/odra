use core::hash::{Hash, Hasher};
use crate::OdraType;
use odra_types::EncodedKeyHash;
use twox_hash::XxHash64;

pub trait Key: Hash {
    fn encoded_hash(&self) -> EncodedKeyHash {
        let mut hasher = XxHash64::default();
        self.hash(&mut hasher);
        let hash = hasher.finish().to_be_bytes();
        let mut key_bytes = [0u8; 16];
        odra_utils::hex_to_slice(&hash, &mut key_bytes);
        key_bytes
    }
}

impl<T: OdraType + Hash> Key for T {}
