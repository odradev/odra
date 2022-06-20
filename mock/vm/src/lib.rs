use ref_thread_local::RefThreadLocal;

use self::vm::MockVm;

mod context;
mod storage;
mod vm;

ref_thread_local::ref_thread_local!(
    static managed ENV: MockVm = MockVm::default();
);

pub fn borrow_env<'a>() -> ref_thread_local::Ref<'a, MockVm> {
    ENV.borrow()
}

pub fn borrow_mut_env<'a>() -> ref_thread_local::RefMut<'a, MockVm> {
    ENV.borrow_mut()
}
