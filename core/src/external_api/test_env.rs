use odra_types::{
    bytesrepr::Bytes, event::Error as EventError, Address, EventData, OdraError, RuntimeArgs,
};

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(contract_name: &str, args: &RuntimeArgs) -> Address {
        odra_test_env_wrapper::on_backend(|env| env.register_contract(contract_name, args))
    }

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

    // @TODO: implement
    pub fn assert_exception<E, F>(err: E, block: F)
    where
        E: Into<OdraError>,
        F: Fn() -> () + std::panic::RefUnwindSafe,
    {
        let _ = std::panic::catch_unwind(|| {
            block();
        });

        let expected: OdraError = err.into();
        let msg = format!("Expected {:?} error.", expected);
        let error: OdraError = TestEnv::get_error().expect(&msg);
        assert_eq!(error, expected);
    }

    pub fn backend_name() -> String {
        odra_test_env_wrapper::on_backend(|env| env.backend_name())
    }

    pub fn set_caller(address: &Address) {
        odra_test_env_wrapper::on_backend(|env| {
            env.set_caller(address);
        });
    }

    pub fn get_account(n: usize) -> Address {
        odra_test_env_wrapper::on_backend(|env| env.get_account(n))
    }

    pub fn get_error() -> Option<OdraError> {
        odra_test_env_wrapper::on_backend(|env| env.get_error())
    }

    pub fn get_event(address: &Address, index: i32) -> Result<EventData, EventError> {
        odra_test_env_wrapper::on_backend(|env| env.get_event(address, index))
    }
}
