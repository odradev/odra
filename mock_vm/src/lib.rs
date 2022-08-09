use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use ref_thread_local::RefThreadLocal;

mod context;
mod contract_container;
mod contract_env;
mod contract_register;
mod mock_vm;
mod storage;
mod test_env;

pub use {contract_env::ContractEnv, test_env::TestEnv};

pub(crate) type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

ref_thread_local::ref_thread_local!(
    static managed ENV: mock_vm::MockVm = mock_vm::MockVm::default();
);

pub(crate) fn borrow_env<'a>() -> ref_thread_local::Ref<'a, mock_vm::MockVm> {
    ENV.borrow()
}
