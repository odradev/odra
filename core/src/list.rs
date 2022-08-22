use core::ops::Range;

use crate::{ContractEnv, Mapping, Variable};
use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, CollectionError
};

use crate::Instance;

/// Data structure for an indexed, iterable collection.
#[derive(Debug)]
pub struct List<T> {
    values: Mapping<u32, T>,
    index: Variable<u32>,
}

impl<T: ToBytes + FromBytes + CLTyped> List<T> {
    /// Creates a new List instance.
    pub fn new(name: String) -> Self {
        let mut name_values = name.clone();
        name_values.push_str("values");
        let mut name_index = name;
        name_index.push_str("index");
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
    pub fn replace(&self, index: u32, value: T) -> Option<T> {
        let current_index = self.index.get_or_default();
        if current_index < index {
            ContractEnv::revert(CollectionError::IndexOutOfBounds);
        }

        let prev_value = self.values.get(&current_index);
        self.values.set(&current_index, value);
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
    T: ToBytes + FromBytes + CLTyped,
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

impl<'a, T> core::iter::ExactSizeIterator for Iter<'a, T> where T: ToBytes + FromBytes + CLTyped {}

impl<'a, T> core::iter::FusedIterator for Iter<'a, T> where T: ToBytes + FromBytes + CLTyped {}

impl<'a, T> core::iter::DoubleEndedIterator for Iter<'a, T>
where
    T: ToBytes + FromBytes + CLTyped,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.range.nth_back(0)?;
        self.list.get(index)
    }
}

impl<T: ToBytes + FromBytes + CLTyped + Default> List<T> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, index: u32) -> T {
        self.get(index).unwrap_or_default()
    }
}

impl<T: ToBytes + FromBytes + CLTyped> From<&str> for List<T> {
    fn from(name: &str) -> Self {
        List::new(String::from(name))
    }
}

impl<T: ToBytes + FromBytes + CLTyped> Instance for List<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}
