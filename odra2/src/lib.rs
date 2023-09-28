#![no_std]

pub use odra_core::{
    mapping::Mapping, module, module::ModuleWrapper, prelude, variable::Variable, ContractEnv, HostEnv
};
pub use odra_types as types;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_backend2;

#[cfg(not(target_arch = "wasm32"))]
mod casper_test {
    use odra_casper_test_env2;
    pub fn test_env() -> odra_core::HostEnv {
        odra_casper_test_env2::CasperVm::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use casper_test::*;