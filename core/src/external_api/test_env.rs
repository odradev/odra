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
    pub fn assert_exception<F, E>(err: E, block: F)
    where
        F: Fn() -> (),
        E: Into<OdraError>,
    {
        // odra_test_env_wrapper::on_backend(|env| {
        //     env.assert_exception(err, block);
        // })
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
}
