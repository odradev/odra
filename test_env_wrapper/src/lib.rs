use std::cell::RefCell;

use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use odra_types::{bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs};

thread_local! {
    static TEST_ENV: RefCell<Container<TestBackend>> = RefCell::new(unsafe {
        Container::load("libodra_test_env.so")
            .expect("Could not open library or load symbols")
    });
}

#[derive(WrapperApi)]
pub struct TestBackend {
    backend_name: fn() -> String,
    register_contract: fn(name: &str, args: &RuntimeArgs) -> Address,
    call_contract:
        fn(addr: &Address, entrypoint: &str, args: &RuntimeArgs, has_return: bool) -> Bytes,
    get_account: fn(n: usize) -> Address,
    set_caller: fn(address: &Address),
    get_error: fn() -> Option<OdraError>,
    get_event: fn(address: &Address, index: i32) -> Result<EventData, EventError>,
}

pub fn on_backend<F, R>(f: F) -> R
where
    F: FnOnce(&Container<TestBackend>) -> R,
{
    TEST_ENV.with(|env| {
        let backend = &*env.borrow();
        f(backend)
    })
}
