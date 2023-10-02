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
    use odra_core::HostContext;

    pub fn test_env() -> odra_core::HostEnv {
        extern crate std;
        let backend = std::env::var("ODRA_BACKEND");
        return match backend {
            Ok(backend_name) => match backend_name.as_str() {
                "casper" => odra_casper_test_env2::CasperVm::new(),
                "odra-vm" | _ => odra_vm_test_env::OdraVm::new()
            },
            Err(_) => odra_vm_test_env::OdraVm::new()
        };
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use odra_test::*;
