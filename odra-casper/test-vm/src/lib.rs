#![doc = "TestVM for Odra Framework."]
#![doc = "It uses CasperVM as a backend for running the tests."]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod casper_host;
mod vm;

pub use casper_host::CasperHost;
pub use vm::casper_vm::CasperVm;
