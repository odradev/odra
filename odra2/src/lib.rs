#![no_std]

extern crate alloc;

pub use odra_core::casper_event_standard;
pub use odra_core::casper_event_standard::Event;
pub use odra_core::event;
pub use odra_core::EntryPointsCaller;
pub use odra_core::{
    call_result::CallResult, mapping::Mapping, module, module::ModuleWrapper, prelude,
    variable::Variable, CallDef, ContractEnv, HostEnv
};
pub use odra_types as types;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_wasm_env;

#[cfg(not(target_arch = "wasm32"))]
mod odra_test {
    use odra_casper_test_vm::{CasperHost, CasperVm};
    use odra_core::prelude::String;
    use odra_core::HostEnv;
    use odra_vm::{OdraVm, OdraVmHost};

    pub fn test_env() -> odra_core::HostEnv {
        extern crate std;
        let backend: String = std::env::var("ODRA_BACKEND").unwrap_or(String::from("odra-vm"));
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
