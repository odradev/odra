use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::OdraType;

pub trait Key: Hash {
    fn encoded_hash(&self) -> [u8; 16] {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        let hash = hasher.finish().to_be_bytes();
        let mut key_bytes = [0u8; 16];
        odra_utils::hex_to_slice(&hash, &mut key_bytes);
        key_bytes
    }
}

impl<T: OdraType + Hash> Key for T {}