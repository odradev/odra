use odra_types::EventData;
use odra_mock_vm::{borrow_env, borrow_mut_env};

pub(crate) fn emit_event(ev: &EventData) {
    borrow_mut_env().emit_event(ev);
}

pub(crate) fn event(at: i32) -> EventData {
    borrow_env().event(at)
}

pub(crate) fn assert_event_type_emitted(event_name: &str) {
    borrow_env().assert_event_type_emitted(event_name);
}

pub(crate) fn assert_event_emitted(event_data: &EventData) {
    borrow_env().assert_event_emitted(event_data);
}

pub(crate) fn assert_event(event_data: &EventData, at: i32) {
    borrow_env().assert_event(event_data, at)
}

pub(crate) fn assert_event_type(event_name: &str, at: i32) {
    borrow_env().assert_event_type(event_name, at)
}

pub(crate) fn assert_event_type_not_emitted(event_name: &str) {
    borrow_env().assert_event_type_not_emitted(event_name)
}

pub(crate) fn assert_event_not_emitted(event_data: &EventData) {
    borrow_env().assert_event_not_emitted(event_data)
}
