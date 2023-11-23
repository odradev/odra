#![no_std]

pub use odra_core::call_result::*;
pub use odra_core::casper_event_standard::Event;
pub use odra_core::mapping::*;
pub use odra_core::module::*;
pub use odra_core::variable::*;
pub use odra_core::*;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_wasm_env;

#[cfg(not(target_arch = "wasm32"))]
pub mod odra_test {
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

    pub fn casper_env() -> HostEnv {
        let vm = CasperVm::new();
        let host_env = CasperHost::new(vm);
        HostEnv::new(host_env)
    }

    pub fn odra_env() -> HostEnv {
        let vm = OdraVm::new();
        let host_env = OdraVmHost::new(vm);
        HostEnv::new(host_env)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use odra_test::*;
