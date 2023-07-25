use crate::UnwrapOrRevert;

use super::{contract_env, types::OdraType};

const KEY_LENGTH: usize = 64;

pub trait KeyResolver<'a> {
    fn resolve(&self) -> alloc::vec::Vec<u8>;

    #[inline]
    fn adjust_key(preimage: &[u8]) -> alloc::vec::Vec<u8> {
        let hash = contract_env::hash(preimage);
        let mut result = [0; KEY_LENGTH];
        odra_utils::hex_to_slice(&hash, &mut result);
        result.to_vec()
    }
}

impl KeyResolver<'_> for &'static[u8] {
    fn resolve(&self) -> alloc::vec::Vec<u8> {
        Self::adjust_key(self)
    }
}

impl<'a, K: OdraType> KeyResolver<'a> for (&'static[u8], &'a K) {
    fn resolve(&self) -> alloc::vec::Vec<u8> {
        let pk = self.1.serialize().unwrap_or_revert();
        let mut storage_key = alloc::vec::Vec::with_capacity(self.0.len() + pk.len());
        storage_key.extend_from_slice(self.0);
        storage_key.extend_from_slice(pk.as_slice());
        Self::adjust_key(&storage_key)
    }
}

impl<'a, P: OdraType, S: OdraType> KeyResolver<'a> for (&'static[u8], &'a P, &'a S) {
    fn resolve(&self) -> alloc::vec::Vec<u8> {
        let pk = self.1.serialize().unwrap_or_revert();
        let sk = self.2.serialize().unwrap_or_revert();
        let mut storage_key = alloc::vec::Vec::with_capacity(self.0.len() + pk.len() + sk.len());
        storage_key.extend_from_slice(self.0);
        storage_key.extend_from_slice(pk.as_slice());
        storage_key.extend_from_slice(sk.as_slice());
        Self::adjust_key(&storage_key)
    }
}

impl<'a, P: OdraType, S: OdraType> KeyResolver<'a> for (&'a P, &'a S) {
    fn resolve(&self) -> alloc::vec::Vec<u8> {
        let pk = self.0.serialize().unwrap_or_revert();
        let sk = self.1.serialize().unwrap_or_revert();
        let mut storage_key = alloc::vec::Vec::with_capacity(pk.len() + sk.len());
        storage_key.extend_from_slice(pk.as_slice());
        storage_key.extend_from_slice(sk.as_slice());
        Self::adjust_key(&storage_key)
    }
}

pub trait Readable {
    fn read<T: OdraType>(&self) -> Option<T>;
}

impl <'a, K: KeyResolver<'a>> Readable for K {
    fn read<T: OdraType>(&self) -> Option<T> {
        let key = self.resolve();
        contract_env::get_var(&key)
    }
}

// impl <K: AsRef<[u8]>> Readable for K {
//     fn read<T: OdraType>(&self) -> Option<T> {
//         let key = self.as_ref();
//         contract_env::get_var(key)
//     }
// }

pub trait Writeable {
    fn write<T: OdraType>(&self, value: T);
}

impl <'a, K: KeyResolver<'a>> Writeable for K {
    fn write<T: OdraType>(&self, value: T) {
        let key = self.resolve();
        contract_env::set_var(&key, value)
    }
}