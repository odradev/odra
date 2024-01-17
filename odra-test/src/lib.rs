#![no_std]

#[cfg(target_arch = "wasm32")]
compile_error!("odra-test is not meant to be compiled for wasm32");

use odra_casper_test_vm::{CasperHost, CasperVm};
use odra_core::prelude::String;
use odra_core::HostEnv;
use odra_vm::{OdraVm, OdraVmHost};

pub fn env() -> odra_core::HostEnv {
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
