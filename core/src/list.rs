use core::ops::Range;

use alloc::rc::Rc;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

use crate::{
    module::{ModuleComponent, ModulePrimitive, Revertible},
    prelude::*,
    CollectionError, ContractEnv
};

/// Data structure for an indexed, iterable collection.
pub struct List<T> {
    env: Rc<ContractEnv>,
    index: u8,
    values: Mapping<u32, T>,
    current_index: Var<u32>
}

impl<T> List<T> {
    /// Returns the ContractEnv.
    pub fn env(&self) -> ContractEnv {
        self.env.child(self.index)
    }
}

impl<T> ModuleComponent for List<T> {
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env: env.clone(),
            index,
            values: Mapping::instance(env.child(index).into(), 0),
            current_index: Var::instance(env.child(index).into(), 1)
        }
    }
}

impl<T> Revertible for List<T> {
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.env.revert(e)
    }
}

impl<T> ModulePrimitive for List<T> {}

impl<T> List<T> {
    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the collection length.
    pub fn len(&self) -> u32 {
        self.current_index.get_or_default()
    }
}

impl<T: FromBytes + CLTyped> List<T> {
    /// Reads collection's n-th value from the storage or returns `None`.
    pub fn get(&self, index: u32) -> Option<T> {
        self.values.get(&index)
    }
}

impl<T: ToBytes + FromBytes + CLTyped> List<T> {
    /// Pushes the `value` to the storage.
    pub fn push(&mut self, value: T) {
        let next_index = self.len();
        self.values.set(&next_index, value);
        self.current_index.set(next_index + 1);
    }

    /// Replaces the current value with the `value` and returns it.
    pub fn replace(&mut self, index: u32, value: T) -> T {
        if index >= self.len() {
            self.env.revert(CollectionError::IndexOutOfBounds);
        }

        let prev_value = self.values.get(&index).unwrap_or_revert(self);
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
        let value = self.values.get(&last).unwrap_or_revert(self);
        self.current_index.set(last);
        Some(value)
    }

    /// Returns an iterator.
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter::new(self)
    }
}

/// An iterator over the elements of a `List`.
///
/// This struct is created by the [`iter`] method on [`List`]. See its documentation for more.
///
/// [`iter`]: struct.List.html#method.iter
/// [`List`]: struct.List.html
pub struct ListIter<'a, T> {
    list: &'a List<T>,
    range: Range<u32>
}

impl<'a, T> ListIter<'a, T> {
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

impl<T> core::iter::Iterator for ListIter<'_, T>
where
    T: ToBytes + FromBytes + CLTyped
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

impl<T> core::iter::ExactSizeIterator for ListIter<'_, T> where T: ToBytes + FromBytes + CLTyped {}

impl<T> core::iter::FusedIterator for ListIter<'_, T> where T: ToBytes + FromBytes + CLTyped {}

impl<T> core::iter::DoubleEndedIterator for ListIter<'_, T>
where
    T: ToBytes + FromBytes + CLTyped
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
