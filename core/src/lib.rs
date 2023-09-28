#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(all(feature = "casper", feature = "mock-vm"))]
compile_error!("casper and mock-vm are mutually exclusive features.");

#[cfg(all(feature = "casper", feature = "casper-livenet"))]
compile_error!("casper and casper-livenet are mutually exclusive features.");

#[cfg(all(feature = "mock-vm", feature = "casper-livenet"))]
compile_error!("mock-vm and casper-livenet are mutually exclusive features.");

#[cfg(all(feature = "casper", feature = "mock-vm", feature = "casper-livenet"))]
compile_error!("mock-vm, casper and casper-livenet are mutually exclusive features.");

#[cfg(not(any(feature = "casper", feature = "mock-vm", feature = "casper-livenet")))]
compile_error!(
    "Exactly one of these features must be selected: `casper`, `mock-vm`, `casper-livenet`."
);

mod instance;
mod item;
mod list;
mod mapping;
#[cfg(not(target_arch = "wasm32"))]
mod node;
mod sequence;
mod unwrap_or_revert;
mod variable;

#[cfg(not(any(target_arch = "wasm32", feature = "casper-livenet")))]
pub mod test_utils;

pub mod prelude {
    #[cfg(feature = "std")]
    pub use std::{borrow, boxed, format, string, vec};

    #[cfg(feature = "std")]
    pub use std::string::ToString;

    #[cfg(feature = "std")]
    pub mod collections {
        pub use self::{
            binary_heap::BinaryHeap, btree_map::BTreeMap, btree_set::BTreeSet,
            linked_list::LinkedList, vec_deque::VecDeque, Bound
        };
        pub use std::collections::*;
    }

    #[cfg(feature = "std")]
    pub use std::cell::RefCell;
    #[cfg(feature = "std")]
    pub use std::rc::Rc;

    #[cfg(not(feature = "std"))]
    pub use alloc::{borrow, boxed, format, string, string::ToString, vec};

    #[cfg(not(feature = "std"))]
    pub mod collections {
        pub use self::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque};
        pub use alloc::collections::*;
        pub use core::ops::Bound;
    }

    #[cfg(not(feature = "std"))]
    pub use alloc::rc::Rc;
    #[cfg(not(feature = "std"))]
    pub use core::cell::RefCell;
}

pub use {
    instance::{DynamicInstance, StaticInstance},
    item::OdraItem,
    list::{Iter, List},
    mapping::Mapping,
    odra_proc_macros::{
        execution_error, external_contract, map, module, odra_error, Event, Instance, OdraType
    },
    odra_utils as utils,
    sequence::Sequence,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable
};

#[cfg(feature = "mock-vm")]
mod env {
    pub use odra_mock_vm::{contract_env, test_env};
    pub mod types {
        pub use odra_types::*;
    }
    pub use test_env::call_contract;
}

// Casper WASM.
#[cfg(all(feature = "casper", target_arch = "wasm32"))]
mod env {
    pub use odra_casper_backend::contract_env;
    pub mod types {
        pub use odra_types::*;
    }
    pub mod casper {
        pub use odra_casper_backend::{casper_contract, contract_env, runtime, storage, utils};
    }
    pub use contract_env::call_contract;
}

// Casper Test.
#[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
mod env {
    pub use odra_casper_test_env::{dummy_contract_env as contract_env, test_env};
    pub mod types {
        pub use odra_types::*;
    }
    pub mod casper {
        pub use odra_casper_codegen as codegen;
    }
    pub use test_env::call_contract;
}

#[cfg(feature = "casper-livenet")]
mod env {
    pub use odra_casper_livenet::{client_env, contract_env};
    pub mod types {
        pub use odra_types::*;
    }
    pub use client_env::call_contract;
}

pub use env::*;
