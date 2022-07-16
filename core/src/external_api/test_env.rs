use odra_types::{bytesrepr::Bytes, Address, EventData, RuntimeArgs, OdraError};

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
        let maybe_error: Option<OdraError> = TestEnv::get_error();
        if let Some(error) = maybe_error {
            let expected: OdraError = err.into();
            assert_eq!(error, expected);
        }
    }

    pub fn backend_name() -> String {
        odra_test_env_wrapper::on_backend(|env| {
            env.backend_name()
        })
    }

    pub fn set_caller(address: &Address) {
        odra_test_env_wrapper::on_backend(|env| {
            env.set_caller(address);
        });
    }

    pub fn get_account(n: usize) -> Address {
        odra_test_env_wrapper::on_backend(|env| {
            env.get_account(n)
        })
    }

    pub fn get_error() -> Option<OdraError> {
        odra_test_env_wrapper::on_backend(|env| {
            env.get_error()
        })
    }
}
