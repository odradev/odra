use core::ops::Range;

use odra_mock_vm::types::OdraType;
use crate::types::odra_types::CollectionError;

use crate::{contract_env, Instance, Mapping, UnwrapOrRevert, Variable};

/// Data structure for an indexed, iterable collection.
pub struct List<T> {
    values: Mapping<u32, T>,
    index: Variable<u32>,
}

impl<T: OdraType> List<T> {
    /// Creates a new List instance.
    pub fn new(name: String) -> Self {
        let mut name_values = name.clone();
        name_values.push_str("_values");
        let mut name_index = name;
        name_index.push_str("_index");
        List {
            values: Mapping::new(name_values),
            index: Variable::new(name_index),
        }
    }

    /// Reads collection's n-th value from the storage or returns `None`.
    pub fn get(&self, index: u32) -> Option<T> {
        self.values.get(&index)
    }

    /// Pushes the `value` to the storage.
    pub fn push(&self, value: T) {
        let current_index = self.index.get_or_default();
        self.values.set(&current_index, value);
        self.index.set(current_index + 1);
    }

    /// Replaces the current value with the `value` and returns it.
    pub fn replace(&self, index: u32, value: T) -> T {
        let current_index = self.index.get_or_default();
        if current_index < index {
            contract_env::revert(CollectionError::IndexOutOfBounds);
        }

        let prev_value = self.values.get(&index).unwrap_or_revert();
        self.values.set(&index, value);
        prev_value
    }

    /// Returns an iterator.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

impl<T> List<T> {
    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.index.get_or_default() == 0
    }

    /// Gets the collection length.
    pub fn len(&self) -> u32 {
        self.index.get_or_default()
    }
}

pub struct Iter<'a, T> {
    list: &'a List<T>,
    range: Range<u32>,
}

impl<'a, T> Iter<'a, T> {
    /// Returns a new instance of Iter.
    fn new(list: &'a List<T>) -> Self {
        Self {
            list,
            range: Range {
                start: 0,
                end: list.len(),
            },
        }
    }

    /// Returns number of elements left to iterate.
    fn remaining(&self) -> usize {
        (self.range.end - self.range.start) as usize
    }
}

impl<'a, T> core::iter::Iterator for Iter<'a, T>
where
    T: OdraType,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.remaining()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.range.nth(n)?;
        self.list.get(index)
    }
}

impl<'a, T> core::iter::ExactSizeIterator for Iter<'a, T> where T: OdraType {}

impl<'a, T> core::iter::FusedIterator for Iter<'a, T> where T: OdraType {}

impl<'a, T> core::iter::DoubleEndedIterator for Iter<'a, T>
where
    T: OdraType,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.nth_back(0)?;
        self.list.get(index)
    }
}

impl<T: OdraType + Default> List<T> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, index: u32) -> T {
        self.get(index).unwrap_or_default()
    }
}

impl<T: OdraType> From<&str> for List<T> {
    fn from(name: &str) -> Self {
        List::new(String::from(name))
    }
}

impl<T: OdraType> Instance for List<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use odra_mock_vm::types::OdraType;
    use crate::types::odra_types::CollectionError;

    use crate::test_env;

    use crate::List;

    #[test]
    fn test_getting_items() {
        // Given an empty list
        let list = List::<u8>::default();
        assert_eq!(list.len(), 0);

        // When push a first item
        list.push(0u8);
        // Then a value at index 0 is available
        assert_eq!(list.get(0).unwrap(), 0);

        // When push next two items
        list.push(1u8);
        list.push(3u8);

        // Then these values are accessible at indexes 1 and 2
        assert_eq!(list.get(1).unwrap(), 1);
        assert_eq!(list.get(2).unwrap(), 3);

        // When get a value under nonexistent index
        let result = list.get(100);
        // Then the value is None
        assert_eq!(result, None);
    }

    #[test]
    fn test_replace() {
        // Given a list with 5 items
        let list = List::<u8>::default();
        for i in 0..5 {
            list.push(i);
        }

        // When replace last item
        let result = list.replace(4, 10);

        // Then the previous value is returned
        assert_eq!(result, 4);
        // Then the value is updated
        assert_eq!(list.get(4).unwrap(), 10);

        // When replaces nonexistent value then reverts
        test_env::assert_exception(CollectionError::IndexOutOfBounds, || {
            list.replace(100, 99);
        });
    }

    #[test]
    fn test_list_len() {
        // Given an empty list
        let list = List::<u8>::default();

        // When push 3 elements
        assert_eq!(list.len(), 0);
        list.push(0u8);
        list.push(1u8);
        list.push(3u8);

        // Then the length should be 3
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_is_empty() {
        // Given an empty list
        let list = List::<u8>::default();
        assert!(list.is_empty());

        // When push an element
        list.push(9u8);

        // Then the list should not be empty
        assert!(!list.is_empty());
    }

    #[test]
    fn test_iter() {
        // Given a list with 5 items
        let list = List::<u8>::default();
        for i in 0..5 {
            list.push(i);
        }

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_fuse_iter() {
        // Given a list with 3 items
        let list = List::<u8>::default();
        for i in 0..3 {
            list.push(i);
        }

        // When iterate over all the elements
        let iter = list.iter();
        let mut iter = iter.fuse();
        iter.next();
        iter.next();
        iter.next();

        // Then all consecutive iter.next() calls return None
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_double_ended_iter() {
        // Given a list with 10 items
        let list = List::<u8>::default();
        for i in 0..10 {
            list.push(i);
        }

        let mut iter = list.iter();

        // When iterate from the start
        // Then first two iterations returns the first and the second item
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        // When iterate from the end
        // Then two iterations returns 10th and 9th items
        assert_eq!(iter.next_back(), Some(9));
        assert_eq!(iter.next_back(), Some(8));
        // When iterate from the start again
        // Then the first iteration returns third element
        assert_eq!(iter.next(), Some(2));
        // Then five items remaining
        assert_eq!(iter.count(), 5);
    }

    impl<T: OdraType> Default for List<T> {
        fn default() -> Self {
            Self::new(String::from("l"))
        }
    }
}
