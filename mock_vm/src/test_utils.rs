use odra_types::{EventData, bytesrepr::FromBytes};

use crate::borrow_env;

pub fn event(at: i32) -> EventData {
    events().get(index_to_usize(at)).unwrap().clone()
}

pub fn assert_event_type_emitted(event_name: &str) {
    for event in events().iter() {
        if get_event_name(event.as_slice()) == *event_name {
            return;
        }
    }

    // TODO: better message
    assert_eq!(event_name, "");
}

pub fn assert_event_emitted(event_data: &EventData) {
    for event in events().iter() {
        if event == event_data {
            return;
        }
    }

    // TODO: better message
    panic!("Event not found")
}

pub fn assert_event(event_data: &EventData, at: i32) {
    assert_eq!(event(at), event_data.clone())
}

pub fn assert_event_type(event_name: &str, at: i32) {
    assert_eq!(
        get_event_name(event(at).as_slice()),
        event_name
    );
}

pub fn assert_event_type_not_emitted(event_name: &str) {
    for event in events().iter() {
        if get_event_name(event.as_slice()) == *event_name {
            // TODO: better message
            assert_eq!(event_name, "");
        }
    }
}

pub fn assert_event_not_emitted(event_data: &EventData) {
    for event in events().iter() {
        if event == event_data {
            // TODO: better message
            panic!("Event not found")
        }
    }
}

fn index_to_usize(index: i32) -> usize {
    if index.is_negative() {
        events().len() - index.wrapping_abs() as usize
    } else {
        index as usize
    }
}

fn get_event_name(event_data: &[u8]) -> String {
    let (name, _): (String, _) = FromBytes::from_bytes(event_data).unwrap();
    name
}

fn events() -> Vec<EventData> {
    borrow_env().events()
}
