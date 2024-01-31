use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, CLValue, RuntimeArgs};


/// A trait for unchecked getters.
pub trait UncheckedGetter {
    /// Retrieves a value of type `T` from the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value from.
    ///
    /// # Returns
    ///
    /// The retrieved value of type `T`.
    ///
    /// # Panics
    ///
    /// This method may panic if the value cannot be retrieved or converted to type `T`.
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T;
}

impl UncheckedGetter for RuntimeArgs {
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T {
        self.get(key)
            .cloned()
            .map(CLValue::into_t)
            .unwrap()
            .unwrap()
    }
}
