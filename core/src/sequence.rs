use crate::{instance::StaticInstance, types::OdraType, Variable};
use num_traits::{Num, One, Zero};

use super::DynamicInstance;

/// A module that stores a single value in the storage that can be read or incremented.
#[derive(Clone)]
pub struct Sequence<T>
where
    T: Num + One + OdraType
{
    value: Variable<T>
}

impl<T> Sequence<T>
where
    T: Num + One + Zero + Default + Copy + OdraType
{
    pub fn get_current_value(&self) -> T {
        self.value.get().unwrap_or_default()
    }

    pub fn next_value(&mut self) -> T {
        match self.value.get() {
            None => {
                self.value.set(T::zero());
                T::zero()
            }
            Some(value) => {
                let next = value + T::one();
                self.value.set(next);
                next
            }
        }
    }
}

impl<T> StaticInstance for Sequence<T>
where
    T: Num + One + OdraType
{
    fn instance(keys: &'static [&'static str]) -> (Self, &'static [&'static str]) {
        let (value, rem) = StaticInstance::instance(keys);
        (Self { value }, rem)
    }
}
impl<T> DynamicInstance for Sequence<T>
where
    T: Num + One + OdraType
{
    fn instance(namespace: &[u8]) -> Self {
        let mut buffer: Vec<u8> = Vec::with_capacity(namespace.len() + b"value".len());
        buffer.extend_from_slice(namespace);
        buffer.extend_from_slice(b"value");
        Self {
            value: DynamicInstance::instance(&buffer)
        }
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::types::U256;
    use crate::{sequence::Sequence, StaticInstance};

    const KEYS_U8: [&str; 1] = ["u8"];
    const KEYS_U256: [&str; 1] = ["u256"];

    #[test]
    fn it_works() {
        let (mut seq_u8, _) = Sequence::<u8>::instance(&KEYS_U8);

        assert_eq!(seq_u8.next_value(), 0u8);
        assert_eq!(seq_u8.next_value(), 1u8);
        assert_eq!(seq_u8.next_value(), 2u8);
        assert_eq!(seq_u8.get_current_value(), 2u8);

        let (mut seq_u256, _) = Sequence::<U256>::instance(&KEYS_U256);

        assert_eq!(seq_u256.next_value(), U256::zero());
        assert_eq!(seq_u256.next_value(), U256::one());
        assert_eq!(seq_u256.next_value(), U256::one() + U256::one());
        assert_eq!(seq_u256.get_current_value(), U256::from(2));
    }
}
