//! Common API for `no_std` and `std` to access alloc and core crate types.
//!
//! Guarantees a stable API for `no_std` and `std` mode.

#[allow(clippy::module_inception)]
mod prelude {
    pub use crate::address::Address;
    pub use crate::arithmetic::*;
    pub use crate::contract_env::ContractEnv;
    pub use crate::error::{ExecutionError, OdraError, OdraResult};
    pub use crate::external::External;
    pub use crate::list::{List, ListIter};
    pub use crate::mapping::Mapping;
    pub use crate::module::{Module, Revertible, SubModule};
    pub use crate::sequence::Sequence;
    pub use crate::unwrap_or_revert::UnwrapOrRevert;
    pub use crate::var::Var;
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::collections::*;
    pub use alloc::format;
    pub use alloc::rc::Rc;
    pub use alloc::slice::Iter;
    pub use alloc::string;
    pub use alloc::string::{FromUtf16Error, FromUtf8Error, ParseError, String, ToString};
    pub use alloc::{vec, vec::*};
    pub use casper_event_standard;
    pub use core::cell::RefCell;
    pub use core::ops::Bound;
    pub use core::str::FromStr;
}

pub use prelude::*;
