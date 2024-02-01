//! This crate provides a testing environment for the Odra VM.
//!
//! It is meant to be used in the unit tests of the Odra contracts.
//!
//! # Example
//!
//! ```no_run
//! #[test]
//! fn test() {
//!    let env = odra_test::env();
//!    let caller = env.get_account(0);
//!
//!    // Test your contract here.
//! }
//! ```
#![no_std]

#[cfg(target_arch = "wasm32")]
compile_error!("odra-test is not meant to be compiled for wasm32");

use odra_casper_test_vm::{CasperHost, CasperVm};
use odra_core::host::HostEnv;
use odra_core::prelude::String;
use odra_vm::{OdraVm, OdraVmHost};

/// Returns the host environment for the testing purpose.
///
/// Two environments are supported: [odra-vm](OdraVmHost) and [casper](CasperHost).
pub fn env() -> HostEnv {
    extern crate std;

    let backend: String = std::env::var("ODRA_BACKEND").unwrap_or_default();
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
