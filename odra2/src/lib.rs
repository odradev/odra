#![no_std]

extern crate alloc;

pub use odra_core::EntryPointsCaller;
pub use odra_core::{
    casper_event_standard, mapping::Mapping, module, module::ModuleWrapper, prelude,
    variable::Variable, CallDef, ContractEnv, HostEnv
};
pub use odra_types as types;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_backend2;

#[cfg(not(target_arch = "wasm32"))]
mod odra_test {
    use alloc::rc::Rc;
    use core::cell::RefCell;
    use odra_casper_test_env2::{CasperHost, CasperVm};
    use odra_core::prelude::String;
    use odra_core::{HostContext, HostEnv};
    use odra_vm::{OdraVm, OdraVmHost};

    pub fn test_env() -> odra_core::HostEnv {
        extern crate std;
        let backend: String = std::env::var("ODRA_BACKEND").unwrap_or(String::from("odra-vm"));

        // let backend = OdraVM::new();
        // let mut contract_env = ContractEnv::new(backend.clone());
        // let test_env = HostEnv::new(backend, contract_env);
        match backend.as_str() {
            "casper" => casper_env(),
            _ => odra_env()
        }
    }

    fn casper_env() -> HostEnv {
        let vm = CasperVm::new();
        let host_env = CasperHost::new(vm);
        HostEnv::new(host_env)
    }

    fn odra_env() -> HostEnv {
        let vm = OdraVm::new();
        let host_env = OdraVmHost::new(vm);
        HostEnv::new(host_env)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use odra_test::*;
