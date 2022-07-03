use ref_thread_local::RefThreadLocal;

mod context;
mod storage;
mod vm;

ref_thread_local::ref_thread_local!(
    static managed ENV: vm::MockVm = vm::MockVm::default();
);

pub fn borrow_env<'a>() -> ref_thread_local::Ref<'a, vm::MockVm> {
    ENV.borrow()
}

pub fn borrow_mut_env<'a>() -> ref_thread_local::RefMut<'a, vm::MockVm> {
    ENV.borrow_mut()
}
