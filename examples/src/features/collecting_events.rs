#![allow(dead_code)]
use casper_event_standard::Event;
use odra::casper_event_standard;
use odra::prelude::*;
use odra::{
    module::{Module, ModuleWrapper},
    Variable
};

#[derive(Event, PartialEq, Eq, Debug)]
struct Start {}

#[derive(Event, PartialEq, Eq, Debug)]
struct Stop {}

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
        self.env().emit_event(Start {});
    }

    pub fn stop(&self) {
        self.env().emit_event(Stop {});
    }
}

#[odra::module(events = [Info])]
struct Machine {
    e1: ModuleWrapper<Engine>,
    e2: ModuleWrapper<Engine>
}

impl Machine {
    pub fn start_first_engine(&self) {
        self.e1.start();
        self.env().emit_event(Info {
            msg: "E1 started".to_string()
        });
    }

    pub fn start_second_engine(&self) {
        self.e2.start();
        self.env().emit_event(Info {
            msg: "E2 started".to_string()
        });
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use odra::casper_types::CLType;
    use odra::contract_def::{Argument, Event, HasEvents};
    use odra::prelude::*;

    use super::{Engine, Machine};

    #[test]
    fn basic_events_collecting_works() {
        let events = <Engine as HasEvents>::events();
        assert_eq!(2, events.len());

        assert_eq!(vec![engine_event("Start"), engine_event("Stop")], events)
    }

    #[test]
    fn nested_events_collecting_works() {
        // collects its own events and children modules events.
        let events = <Machine as HasEvents>::events();
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
