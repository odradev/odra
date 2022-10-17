use odra_types::{
    bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs, U512,
};

macro_rules! delegate_to_wrapper {
    (
        $(
            $(#[$outer:meta])*
            fn $func_name:ident($( $param_ident:ident : $param_ty:ty ),*) $( -> $ret:ty)*
        )+
    ) => {
        $(
            $(#[$outer])*
            pub fn $func_name( $($param_ident : $param_ty),* ) $(-> $ret)* {
                odra_test_env_wrapper::on_backend(|env| env.$func_name($($param_ident),*))
            }
        )+
    }
}

/// Describes test environment API. TestEnv delegates methods to the underlying dynamically loaded env.
///
/// The actual test env is dynamically loaded in the runtime.
pub struct TestEnv;

impl TestEnv {
    delegate_to_wrapper! {
        /// Calls contract at `address` invoking the `entrypoint` with `args`.
        ///
        /// Returns optional raw bytes to further processing.
        fn call_contract(
            address: &Address,
            entrypoint: &str,
            args: &RuntimeArgs,
            has_return: bool,
            amount: Option<U512>
        ) -> Option<Bytes>
        ///Registers the contract in the test environment.
        fn register_contract(contract_name: &str, args: &RuntimeArgs) -> Address
        ///Returns the backend name.
        fn backend_name() -> String
        ///Replaces the current caller.
        fn set_caller(address: &Address)
        ///Returns nth test user account.
        fn get_account(n: usize) -> Address
        ///Gets nth event emitted by the contract at `address`.
        fn get_event(address: &Address, index: i32) -> Result<EventData, EventError>
        ///Gets the current error from the test environment.
        fn get_error() -> Option<OdraError>
        ///Increases the current value of block_time.
        fn advance_block_time_by(seconds: u64)
        /// Returns the balance of the account associated with the given address.
        fn token_balance(address: Address) -> U512
        /// Returns the value that represents one native token.
        fn one_token() -> U512
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
}
