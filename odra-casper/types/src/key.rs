use core::hash::Hasher;

use alloc::{string::String, vec::Vec, collections::BTreeMap};
use casper_types::bytesrepr::Bytes;
use twox_hash::XxHash64;
use crate::{Address, U512, U256, U128};

pub trait Key {
    fn hash(&self) -> [u8; 8];
}

impl Key for Address {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        match self {
            Address::Account(account) => hasher.write(account.as_bytes()),
            Address::Contract(contract) => hasher.write(contract.as_bytes()),
        }
        let result = hasher.finish();
        result.to_le_bytes()
    }
}

impl Key for u8 {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_u8(*self);
        hasher.finish().to_le_bytes()
    }
}

impl Key for u32 {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_u32(*self);
        hasher.finish().to_le_bytes()
    }
}

impl Key for u64 {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_u64(*self);
        hasher.finish().to_le_bytes()
    }
}

impl Key for i32 {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_i32(*self);
        hasher.finish().to_le_bytes()
    }
}

impl Key for i64 {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_i64(*self);
        hasher.finish().to_le_bytes()
    }
}

impl Key for String {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(self.as_bytes());
        hasher.finish().to_le_bytes()
    }
}

impl Key for () {
    fn hash(&self) -> [u8; 8] {
        let hasher = XxHash64::with_seed(0);
        hasher.finish().to_le_bytes()
    }
}

impl<K: Key> Key for Option<K> {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        match self {
            Some(val) => {
                hasher.write_u8(1);
                hasher.write(&val.hash());
            },
            None => hasher.write_u8(0),
        }
        hasher.finish().to_le_bytes()
    }
}

impl<K: Key> Key for Vec<K> {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_usize(self.len());
        self.iter().for_each(|v| hasher.write(&v.hash()));
        hasher.finish().to_le_bytes()
    }
}

impl<K: Key, V: Key> Key for Result<K, V> {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        match self {
            Ok(res) => {
                hasher.write_u8(1);
                hasher.write(&res.hash());
            },
            Err(err) => {
                hasher.write_u8(0);
                hasher.write(&err.hash());
            }
        }
        hasher.finish().to_le_bytes()
    }
}

impl<K: Key, V: Key> Key for BTreeMap<K, V> {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_usize(self.len());
        for (key, value) in self.iter() {
            hasher.write(&key.hash());
            hasher.write(&value.hash());
        }
        hasher.finish().to_le_bytes()
    }
}


impl<const COUNT: usize> Key for [u8; COUNT] {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_usize(self.len());
        self.iter().for_each(|i| hasher.write_u8(*i));
        hasher.finish().to_le_bytes()
    }
}

impl<T1: Key> Key for (T1,) {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&self.0.hash());
        hasher.finish().to_le_bytes()
    }
}

impl<T1: Key, T2: Key> Key for (T1, T2) {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&self.0.hash());
        hasher.write(&self.1.hash());
        hasher.finish().to_le_bytes()
    }
}

impl<T1: Key, T2: Key, T3: Key> Key for (T1, T2, T3) {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&self.0.hash());
        hasher.write(&self.1.hash());
        hasher.write(&self.2.hash());
        hasher.finish().to_le_bytes()
    }
}

macro_rules! impl_key_for_big_int {
    ($($t:ty),+)  => {
        $(
            impl Key for $t {
                fn hash(&self) -> [u8; 8] {
                    let mut hasher = XxHash64::with_seed(0);
                    self.inner().0.iter().for_each(|i| hasher.write_u64(*i));
                    hasher.finish().to_le_bytes()
                }
            }
        )+
    };
}

impl_key_for_big_int!(U512, U256, U128);

impl Key for Bytes {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write_usize(self.len());
        self.inner_bytes().iter().for_each(|i| hasher.write_u8(*i));
        hasher.finish().to_le_bytes()
    }
} 

impl Key for bool {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        <bool as core::hash::Hash>::hash(self, &mut hasher);
        hasher.finish().to_le_bytes()
    }
}