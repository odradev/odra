use std::ops::Range;

use num_traits::{Num, One};
use crate::types::OdraType;

use crate::{Variable, Mapping, List, Sequence};

pub trait Node {
	fn count(&self) -> u32;
	fn keys(&self) -> Vec<String>;
	fn is_leaf(&self) -> bool;
    fn key_ranges(&self) -> Vec<Range<u32>>;
    fn range(&self) -> Range<u32>;
}

impl <T> Node for Variable<T> {
    fn count(&self) -> u32 {
        1
    }

    fn keys(&self) -> Vec<String> {
        vec![self.path().to_owned()]
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn key_ranges(&self) -> Vec<Range<u32>> {
        vec![0..1]
    }

    fn range(&self) -> Range<u32> {
        0..1
    }
}

impl <K, V> Node for Mapping<K, V> {
    fn count(&self) -> u32 {
        1
    }

    fn keys(&self) -> Vec<String> {
        vec![self.path().to_owned()]
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn key_ranges(&self) -> Vec<Range<u32>> {
        vec![0..1]
    }

    fn range(&self) -> Range<u32> {
        0..1
    }
}

impl <T> Node for List<T> {
    fn count(&self) -> u32 {
        2
    }

    fn keys(&self) -> Vec<String> {
        todo!()
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn key_ranges(&self) -> Vec<Range<u32>> {
        vec![
            0..1,
            1..2
        ]
    }

    fn range(&self) -> Range<u32> {
        0..2
    }
}

impl <T: Num + One + OdraType> Node for Sequence<T> {
    fn count(&self) -> u32 {
        1
    }

    fn keys(&self) -> Vec<String> {
        vec!["value".to_string()]
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn key_ranges(&self) -> Vec<Range<u32>> {
        vec![0..1]
    }

    fn range(&self) -> Range<u32> {
        0..1
    }
}