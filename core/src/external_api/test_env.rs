use odra_types::{Address, EventData};

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
