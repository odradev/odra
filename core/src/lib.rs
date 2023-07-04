mod instance;
mod item;
mod list;
mod mapping;
mod node;
mod sequence;
mod unwrap_or_revert;
mod variable;

#[cfg(not(any(target_arch = "wasm32", odra_backend = "casper-livenet")))]
pub mod test_utils;

pub use {
    item::OdraItem,
    instance::{DynamicInstance, StaticInstance},
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

#[cfg(odra_backend = "mock-vm")]
mod env {
    pub use odra_mock_vm::{contract_env, test_env};
    pub mod types {
        pub use odra_mock_vm::types::*;
        pub use odra_types::*;
    }
    pub use test_env::call_contract;
}

// Casper WASM.
#[cfg(all(odra_backend = "casper", target_arch = "wasm32"))]
mod env {
    pub use odra_casper_backend::contract_env;
    pub mod types {
        pub use odra_casper_types::*;
        pub use odra_types::*;
    }
    pub mod casper {
        pub use odra_casper_backend::{casper_contract, contract_env, runtime, storage, utils};
        pub use odra_casper_types::casper_types;
    }
    pub use contract_env::call_contract;
}

// Casper Test.
#[cfg(all(odra_backend = "casper", not(target_arch = "wasm32")))]
mod env {
    pub use odra_casper_test_env::{dummy_contract_env as contract_env, test_env};
    pub mod types {
        pub use odra_casper_types::*;
        pub use odra_types::*;
    }
    pub mod casper {
        pub use odra_casper_codegen as codegen;
        pub use odra_casper_types::casper_types;
    }
    pub use test_env::call_contract;
}

#[cfg(odra_backend = "casper-livenet")]
mod env {
    pub use odra_casper_livenet::{client_env, contract_env};
    pub mod types {
        pub use odra_casper_types::*;
        pub use odra_types::*;
    }
    pub mod casper {
        pub use odra_casper_types::casper_types;
    }
    pub use client_env::call_contract;
}

pub use env::*;
