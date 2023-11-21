#[cfg(feature = "std")]
pub use std::{borrow, boxed, format, string, vec};

#[cfg(feature = "std")]
pub use std::string::ToString;

#[cfg(feature = "std")]
pub mod collections {
    pub use self::{
        binary_heap::BinaryHeap, btree_map::BTreeMap, btree_set::BTreeSet, linked_list::LinkedList,
        vec_deque::VecDeque, Bound
    };
    pub use std::collections::*;
}

#[cfg(feature = "std")]
pub use std::cell::RefCell;
#[cfg(feature = "std")]
pub use std::rc::Rc;

#[cfg(not(feature = "std"))]
mod prelude {
    pub use alloc::rc::Rc;
    pub use core::cell::RefCell;
    pub mod collections {
        pub use self::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
        pub use alloc::collections::*;
        pub use core::ops::Bound;
    }
    pub use crate::module::Module;
    pub use alloc::string::String;
    pub use alloc::vec;
    pub use alloc::{borrow, boxed, format, string, string::ToString};
    pub use vec::Vec;
}

#[cfg(not(feature = "std"))]
pub use prelude::*;
