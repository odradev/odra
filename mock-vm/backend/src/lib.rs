use ref_thread_local::RefThreadLocal;

mod balance;
mod callstack;
mod contract_container;
pub mod contract_env;
mod contract_register;
mod mock_vm;
mod storage;
pub mod test_env;

pub use contract_container::{EntrypointArgs, EntrypointCall};

ref_thread_local::ref_thread_local!(
    static managed ENV: mock_vm::MockVm = mock_vm::MockVm::default();
);

pub(crate) fn borrow_env<'a>() -> ref_thread_local::Ref<'a, mock_vm::MockVm> {
    ENV.borrow()
}

pub use odra_mock_vm_types as types;