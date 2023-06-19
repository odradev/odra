use crate::types::OdraType;
use num_traits::{Num, One};
use odra_types::contract_def::Node;

use crate::{List, Mapping, Sequence, Variable};

impl<T> Node for Variable<T> {
    fn count() -> u32 {
        1
    }

    fn keys() -> Vec<String> {
        vec![]
    }

    fn is_leaf() -> bool {
        true
    }
}

impl<K, V> Node for Mapping<K, V> {
    fn count() -> u32 {
        1
    }

    fn keys() -> Vec<String> {
        vec![]
    }

    fn is_leaf() -> bool {
        true
    }
}

impl<T> Node for List<T> {
    fn count() -> u32 {
        2
    }

    fn keys() -> Vec<String> {
        vec![String::from("values"), String::from("index")]
    }

    fn is_leaf() -> bool {
        false
    }
}

impl<T: Num + One + OdraType> Node for Sequence<T> {
    fn count() -> u32 {
        1
    }

    fn keys() -> Vec<String> {
        vec!["value".to_string()]
    }

    fn is_leaf() -> bool {
        false
    }
}
