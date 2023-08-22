use crate::{
    prelude::{string::String, vec, vec::Vec},
    types::OdraType,
    List, Mapping, Sequence, Variable
};
use num_traits::{Num, One};
use odra_types::contract_def::Node;

impl<T> Node for Variable<T> {
    const COUNT: u32 = 1;
}

impl<K, V> Node for Mapping<K, V> {
    const COUNT: u32 = 1;
}

impl<T> Node for List<T> {
    const COUNT: u32 = 2;
    const IS_LEAF: bool = false;

    fn __keys() -> Vec<String> {
        vec![String::from("values"), String::from("index")]
    }
}

impl<T: Num + One + OdraType> Node for Sequence<T> {
    const COUNT: u32 = 1;
    const IS_LEAF: bool = false;

    fn __keys() -> Vec<String> {
        vec![String::from("value")]
    }
}
