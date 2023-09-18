use core::ops::Range;
use odra_types::{CollectionError, casper_types::bytesrepr::{ToBytes, FromBytes}};

use crate::{
    contract_env,
    instance::{DynamicInstance, StaticInstance},
    mapping::Mapping,
    prelude::vec::Vec,
    variable::Variable,
    UnwrapOrRevert
};

/// Data structure for an indexed, iterable collection.
#[derive(Clone)]
pub struct List<T> {
    values: Mapping<u32, T>,
    index: Variable<u32>
}

impl<T: DynamicInstance> List<T> {
    pub fn get_instance(&mut self, index: u32) -> T {
        let len = self.len();
        // Clippy doesn't like the following code, but it's the most efficient way to do it.
        // See https://rust-lang.github.io/rust-clippy/master/index.html#/comparison_chain
        #[allow(clippy::comparison_chain)]
        if index == len {
            self.index.set(index + 1);
        } else if index > len {
            contract_env::revert(CollectionError::IndexOutOfBounds);
        }
        self.values.get_instance(&index)
    }

    pub fn next_instance(&mut self) -> T {
        self.get_instance(self.len())
    }
}

impl<T> List<T> {
    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the collection length.
    pub fn len(&self) -> u32 {
        self.index.get_or_default()
    }
}

impl<T: ToBytes + FromBytes> List<T> {
    /// Reads collection's n-th value from the storage or returns `None`.
    pub fn get(&self, index: u32) -> Option<T> {
        self.values.get(&index)
    }

    /// Pushes the `value` to the storage.
    pub fn push(&mut self, value: T) {
        let next_index = self.len();
        self.values.set(&next_index, value);
        self.index.set(next_index + 1);
    }

    /// Replaces the current value with the `value` and returns it.
    pub fn replace(&mut self, index: u32, value: T) -> T {
        if index >= self.len() {
            contract_env::revert(CollectionError::IndexOutOfBounds);
        }

        let prev_value = self.values.get(&index).unwrap_or_revert();
        self.values.set(&index, value);
        prev_value
    }

    /// Pops the last value from the storage or returns `None`.
    pub fn pop(&mut self) -> Option<T> {
        let next_index = self.len();
        if next_index == 0 {
            return None;
        }
        let last = next_index - 1;
        let value = self.values.get(&last).unwrap_or_revert();
        self.index.set(last);
        Some(value)
    }

    /// Returns an iterator.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

pub struct Iter<'a, T> {
    list: &'a List<T>,
    range: Range<u32>
}

impl<'a, T> Iter<'a, T> {
    /// Returns a new instance of Iter.
    fn new(list: &'a List<T>) -> Self {
        Self {
            list,
            range: Range {
                start: 0,
                end: list.len()
            }
        }
    }

    /// Returns number of elements left to iterate.
    fn remaining(&self) -> usize {
        (self.range.end - self.range.start) as usize
    }
}

impl<'a, T> core::iter::Iterator for Iter<'a, T>
where
    T: ToBytes + FromBytes
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

impl<'a, T> core::iter::ExactSizeIterator for Iter<'a, T> where T: ToBytes + FromBytes {}

impl<'a, T> core::iter::FusedIterator for Iter<'a, T> where T: ToBytes + FromBytes {}

impl<'a, T> core::iter::DoubleEndedIterator for Iter<'a, T>
where
    T: ToBytes + FromBytes
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.nth_back(0)?;
        self.list.get(index)
    }
}

impl<T: ToBytes + FromBytes + Default> List<T> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, index: u32) -> T {
        self.get(index).unwrap_or_default()
    }
}

impl<T> StaticInstance for List<T> {
    fn instance<'a>(keys: &'a [&'a str]) -> (Self, &'a [&'a str]) {
        let (values, keys) = StaticInstance::instance(keys);
        let (index, keys) = StaticInstance::instance(keys);
        (Self { values, index }, keys)
    }
}

impl<T> DynamicInstance for List<T> {
    fn instance(namespace: &[u8]) -> Self {
        let namespace_len = namespace.len();
        let max_len = namespace_len + b"values".len();
        let mut buffer: Vec<u8> = Vec::with_capacity(max_len);

        buffer.extend_from_slice(namespace);
        buffer.extend_from_slice(b"values");
        let values = DynamicInstance::instance(&buffer);

        buffer.clear();
        buffer.extend_from_slice(namespace);
        buffer.extend_from_slice(b"index");
        let index = DynamicInstance::instance(&buffer);

        Self { values, index }
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use super::List;
    use crate::{instance::StaticInstance, test_env};
    use odra_types::{CollectionError, casper_types::bytesrepr::{ToBytes, FromBytes}};

    #[test]
    fn test_getting_items() {
        // Given an empty list
        let mut list = List::<u8>::default();
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
        let mut list = List::<u8>::default();
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
        let mut list = List::<u8>::default();

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
        let mut list = List::<u8>::default();
        assert!(list.is_empty());

        // When push an element
        list.push(9u8);

        // Then the list should not be empty
        assert!(!list.is_empty());
    }

    #[test]
    fn test_pop() {
        // Given list with 2 elements.
        let mut list = List::<u8>::default();
        list.push(1u8);
        list.push(2u8);

        // When pop an element
        let result = list.pop();

        // Then the result is the last element
        assert_eq!(result, Some(2));
        // And the length is 1
        assert_eq!(list.len(), 1);

        // When pop another element
        let result = list.pop();

        // Then the result is the last element
        assert_eq!(result, Some(1));
        // And the length is 0
        assert_eq!(list.len(), 0);

        // When pop another element
        let result = list.pop();

        // Then the result is None
        assert_eq!(result, None);
    }

    #[test]
    fn test_iter() {
        // Given a list with 5 items
        let mut list = List::<u8>::default();
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
        let mut list = List::<u8>::default();
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
        let mut list = List::<u8>::default();
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

    impl<T: ToBytes + FromBytes> Default for List<T> {
        fn default() -> Self {
            StaticInstance::instance(&["list_val", "list_idx"]).0
        }
    }
}
