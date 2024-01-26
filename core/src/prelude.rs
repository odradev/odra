#[allow(clippy::module_inception)]
mod prelude {
    pub use crate::arithmetic::*;
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::collections::*;
    pub use alloc::format;
    pub use alloc::rc::Rc;
    pub use alloc::string;
    pub use alloc::string::{String, ToString, FromUtf8Error, FromUtf16Error, ParseError};
    pub use alloc::{vec, vec::*};
    pub use core::cell::RefCell;
    pub use core::ops::Bound;
    pub use core::str::FromStr;
}

pub use prelude::*;
