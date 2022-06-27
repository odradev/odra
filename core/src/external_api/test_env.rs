use odra_test_env::ContractContainer;
use odra_types::{bytesrepr::Bytes, Address, EventData, RuntimeArgs};

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(container: &ContractContainer) -> Address {
        odra_test_env_wrapper::on_backend(|env| env.register_contract(container))
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

    fn set_caller(address: &Address) {
        todo!()
    }

    fn emit_event(event: &EventData) {
        todo!()
    }

    fn assert_event(event: &EventData, at: i32) {
        todo!()
    }

    fn assert_event_type_emitted(event_name: &str) {
        todo!()
    }

    fn assert_event_emitted(event: &EventData) {
        todo!()
    }

    fn event(at: i32) -> EventData {
        todo!()
    }

    fn assert_event_type(event_name: &str, at: i32) {
        todo!()
    }

    fn assert_event_type_not_emitted(event_name: &str) {
        todo!()
    }

    fn assert_event_not_emitted(event: &EventData) {
        todo!()
    }
}
