use crate::{types::OdraType, Instance, UnwrapOrRevert, Variable};
use num_traits::{Num, One, Zero};

/// A module that stores a single value in the storage that can be read or incremented.
#[derive(PartialEq, Eq, Debug)]
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
        self.value.get().unwrap_or_revert()
    }

    pub fn next_value(&mut self) -> T {
        match self.value.get() {
            None => {
                self.value.set(&T::zero());
                T::zero()
            }
            Some(value) => {
                let next = value + T::one();
                self.value.set(&next);
                next
            }
        }
    }
}

impl<T> Instance for Sequence<T>
where
    T: Num + One + OdraType
{
    fn instance(namespace: &str) -> Self {
        Self {
            value: Instance::instance(format!("{}_{}", "value", namespace).as_str())
        }
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::types::U256;
    use crate::{sequence::Sequence, Instance};

    #[test]
    fn it_works() {
        let mut seq_u8 = Sequence::<u8>::instance("u8");

        assert_eq!(seq_u8.next_value(), 0u8);
        assert_eq!(seq_u8.next_value(), 1u8);
        assert_eq!(seq_u8.next_value(), 2u8);
        assert_eq!(seq_u8.get_current_value(), 2u8);

        let mut seq_u256 = Sequence::<U256>::instance("u256");

        assert_eq!(seq_u256.next_value(), U256::zero());
        assert_eq!(seq_u256.next_value(), U256::one());
        assert_eq!(seq_u256.next_value(), U256::one() + U256::one());
        assert_eq!(seq_u256.get_current_value(), U256::from(2));
    }
}
