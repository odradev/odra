use std::cell::RefCell;

use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use odra_types::{bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs, U512};

thread_local! {
    static TEST_ENV: RefCell<Container<TestBackend>> = RefCell::new(unsafe {
        let filename = format!(
            r"{}odra_test_env.{}",
            dlopen::utils::PLATFORM_FILE_PREFIX,
            dlopen::utils::PLATFORM_FILE_EXTENSION
        );
        Container::load(filename).expect("Could not open library or load symbols")
    });
}

/// Wrapped Test Env API.
#[derive(WrapperApi)]
pub struct TestBackend {
    /// Returns the backend name.
    backend_name: fn() -> String,
    /// Registers the contract in Test Env.
    register_contract: fn(name: &str, args: &RuntimeArgs) -> Address,
    /// Calls contract at `address` invoking the `entrypoint` with `args`.
    call_contract:
        fn(addr: &Address, entrypoint: &str, args: &RuntimeArgs, has_return: bool) -> Bytes,
    /// Returns nth user account.
    get_account: fn(n: usize) -> Address,
    /// Replaces the current caller.
    set_caller: fn(address: &Address),
    /// Gets the current error.
    get_error: fn() -> Option<OdraError>,
    /// Gets nth event emitted by contract at `address`.
    get_event: fn(address: &Address, index: i32) -> Result<EventData, EventError>,
    /// Increases the current value of block_time.
    advance_block_time_by: fn(seconds: u64),
    /// Attaches [amount] of native token to the next contract call.
    with_tokens: fn(amount: U512),
    /// Returns the balance of the account associated with the given address.
    token_balance: fn(address: Address) -> U512,
    /// Returns the value that represents one native token.
    one_token: fn() -> U512,
}

/// An entry point for communication with dynamically loaded Test Env.
pub fn on_backend<F, R>(f: F) -> R
where
    F: FnOnce(&Container<TestBackend>) -> R,
{
    TEST_ENV.with(|env| {
        let backend = &*env.borrow();
        f(backend)
    })
}
