use odra_types::{bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs};

/// Describes test environment API. TestEnv delegates methods to the underlying dynamically loaded env.
///
/// The actual test env is dynamically loaded in the runtime.
pub struct TestEnv;

impl TestEnv {
    /// Registers the contract in the test environment.
    pub fn register_contract(contract_name: &str, args: &RuntimeArgs) -> Address {
        odra_test_env_wrapper::on_backend(|env| env.register_contract(contract_name, args))
    }

    /// Calls contract at `address` invoking the `entrypoint` with `args`.
    ///
    /// Returns optional raw bytes to further processing.
    pub fn call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        has_return: bool,
    ) -> Option<Bytes> {
        odra_test_env_wrapper::on_backend(|env| {
            Some(env.call_contract(address, entrypoint, args, has_return))
        })
    }

    /// Expects the `block` execution will fail with the specific error.
    pub fn assert_exception<E, F>(err: E, block: F)
    where
        E: Into<OdraError>,
        F: Fn() + std::panic::RefUnwindSafe,
    {
        let _ = std::panic::catch_unwind(|| {
            block();
        });

        let expected: OdraError = err.into();
        let msg = format!("Expected {:?} error.", expected);
        let error: OdraError = Self::get_error().expect(&msg);
        assert_eq!(error, expected);
    }

    /// Returns the backend name.
    pub fn backend_name() -> String {
        odra_test_env_wrapper::on_backend(|env| env.backend_name())
    }

    /// Replaces the current caller.
    pub fn set_caller(address: &Address) {
        odra_test_env_wrapper::on_backend(|env| {
            env.set_caller(address);
        });
    }

    /// Returns nth test user account.
    pub fn get_account(n: usize) -> Address {
        odra_test_env_wrapper::on_backend(|env| env.get_account(n))
    }

    /// Gets nth event emitted by the contract at `address`.
    pub fn get_event(address: &Address, index: i32) -> Result<EventData, EventError> {
        odra_test_env_wrapper::on_backend(|env| env.get_event(address, index))
    }

    /// Gets the current error from the test environment.
    fn get_error() -> Option<OdraError> {
        odra_test_env_wrapper::on_backend(|env| env.get_error())
    }
}
