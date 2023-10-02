#![no_std]

extern crate alloc;

pub use odra_core::{
    mapping::Mapping, module, module::ModuleWrapper, prelude, variable::Variable, CallDef,
    ContractEnv, HostEnv
};
pub use odra_types as types;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_backend2;

#[cfg(not(target_arch = "wasm32"))]
mod odra_test {
    use odra_core::prelude::String;
    use odra_core::HostContext;

    pub fn test_env() -> odra_core::HostEnv {
        extern crate std;
        let backend: String = std::env::var("ODRA_BACKEND").unwrap_or(String::from("odra-vm"));

        // let backend = OdraVM::new();
        // let mut contract_env = ContractEnv::new(backend.clone());
        // let test_env = HostEnv::new(backend, contract_env);
        match backend.as_str() {
            "casper" => odra_casper_test_env2::CasperVm::new(),
            _ => odra_vm_test_env::OdraVmEnv::new()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use odra_test::*;
