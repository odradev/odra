use ref_thread_local::RefThreadLocal;

mod context;
mod contract_container;
mod contract_env;
mod contract_register;
mod mock_vm;
mod storage;
mod test_env;
pub mod test_utils;

pub use {contract_container::EntrypointCall, contract_env::ContractEnv, test_env::TestEnv};

ref_thread_local::ref_thread_local!(
    static managed ENV: mock_vm::MockVm = mock_vm::MockVm::default();
);

pub fn borrow_env<'a>() -> ref_thread_local::Ref<'a, mock_vm::MockVm> {
    ENV.borrow()
}
