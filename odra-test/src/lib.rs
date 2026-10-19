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
#[cfg(target_arch = "wasm32")]
compile_error!("odra-test is not meant to be compiled for wasm32");

use odra_casper_test_vm::{CasperHost, CasperVm};
use odra_core::host::HostEnv;
use odra_vm::{OdraVm, OdraVmHost};

/// Returns the host environment for the testing purpose.
///
/// Two environments are supported: [odra-vm](OdraVmHost) and [casper](CasperHost).
/// The backend is selected with the `ODRA_BACKEND` env variable (`casper` or unset for OdraVM),
/// which `cargo odra test -b <backend>` sets for you.
///
/// To pin a test to one backend regardless of `ODRA_BACKEND`, use [odra_env] or [casper_env]
/// directly.
pub fn env() -> HostEnv {
    let backend = std::env::var("ODRA_BACKEND").unwrap_or_default();
    match backend.as_str() {
        "casper" => casper_env(),
        _ => odra_env()
    }
}

/// Returns the [CasperHost] environment, ignoring `ODRA_BACKEND`.
///
/// Requires the contracts to be built as wasm (`cargo odra test -b casper`).
pub fn casper_env() -> HostEnv {
    let vm = CasperVm::new();
    let host_env = CasperHost::new(vm);
    HostEnv::new(host_env)
}

/// Returns the [OdraVmHost] environment, ignoring `ODRA_BACKEND`.
///
/// Useful for testing a submodule that is not registered as a contract in `Odra.toml`:
/// such tests run on OdraVM even under `cargo odra test -b casper`.
pub fn odra_env() -> HostEnv {
    let vm = OdraVm::new();
    let host_env = OdraVmHost::new(vm);
    HostEnv::new(host_env)
}
