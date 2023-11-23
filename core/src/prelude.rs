#[allow(clippy::module_inception)]
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

pub use prelude::*;
