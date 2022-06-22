use odra_test_env::ContractContainer;
use odra_types::{Address, EventData, RuntimeArgs, bytesrepr::Bytes};

#[allow(improper_ctypes)]
extern "C" {
    fn set_caller(address: &Address);
    fn emit_event(event: &EventData);
    fn assert_event(event: &EventData, at: i32);
    fn assert_event_type_emitted(event_name: &str);
    fn assert_event_emitted(event: &EventData);
    fn event(at: i32) -> EventData;
    fn assert_event_type(event_name: &str, at: i32);
    fn assert_event_type_not_emitted(event_name: &str);
    fn assert_event_not_emitted(event: &EventData);
}

pub struct TestEnv;

impl TestEnv {

    pub fn register_contract(container: &ContractContainer) -> Address {
        todo!()
    }

    pub fn call_contract(address: &Address, entrypoint: &str, args: &RuntimeArgs, _has_return: bool) -> Option<Bytes> {
        todo!()
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
