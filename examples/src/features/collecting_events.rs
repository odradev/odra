#![allow(dead_code)]

use odra::{
    prelude::string::{String, ToString},
    types::event::OdraEvent,
    Event, Variable
};

#[derive(Event, PartialEq, Eq, Debug)]
struct Start;

#[derive(Event, PartialEq, Eq, Debug)]
struct Stop;

#[derive(Event, PartialEq, Eq, Debug)]
struct Info {
    msg: String
}

#[odra::module(events = [Start, Stop])]
struct Engine {
    name: Variable<String>
}

impl Engine {
    pub fn start(&self) {
        Start.emit();
    }

    pub fn stop(&self) {
        Stop.emit();
    }
}

#[odra::module(events = [Info])]
struct Machine {
    e1: Engine,
    e2: Engine
}

impl Machine {
    pub fn start_first_engine(&self) {
        self.e1.start();
        Info {
            msg: "E1 started".to_string()
        }
        .emit();
    }

    pub fn start_second_engine(&self) {
        self.e2.start();
        Info {
            msg: "E2 started".to_string()
        }
        .emit();
    }
}

#[cfg(all(test, feature = "mock-vm"))]
mod test {
    use odra::{
        prelude::{string::ToString, vec},
        types::{
            contract_def::{Argument, Event},
            CLType
        }
    };

    use super::{Engine, Machine};

    #[test]
    fn basic_events_collecting_works() {
        let events = <Engine as odra::types::contract_def::HasEvents>::events();
        assert_eq!(2, events.len());

        assert_eq!(vec![engine_event("Start"), engine_event("Stop")], events)
    }

    #[test]
    fn nested_events_collecting_works() {
        // collects its own events and children modules events.
        let events = <Machine as odra::types::contract_def::HasEvents>::events();
        assert_eq!(3, events.len());

        assert_eq!(
            vec![info_event(), engine_event("Start"), engine_event("Stop")],
            events
        )
    }

    fn engine_event(ident: &str) -> Event {
        Event {
            ident: ident.to_string(),
            args: vec![]
        }
    }

    fn info_event() -> Event {
        let arg = Argument {
            ident: "msg".to_string(),
            ty: CLType::String,
            is_ref: false,
            is_slice: false
        };
        Event {
            ident: "Info".to_string(),
            args: vec![arg]
        }
    }
}
