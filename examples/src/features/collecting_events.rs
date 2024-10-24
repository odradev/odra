//! This example demonstrates how to collect events from a module and its submodules.
#![allow(dead_code)]
use odra::prelude::*;

#[odra::event]
struct Start {}

#[odra::event]
struct Stop {}

#[odra::event]
struct Info {
    msg: String
}

#[odra::module(
    name = "EngineContract",
    version = "1.0.1",
    events = [Start, Stop])
]
struct Engine {
    name: Var<String>
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
    e1: SubModule<Engine>,
    e2: SubModule<Engine>
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
            name: ident.to_string(),
            args: vec![]
        }
    }

    fn info_event() -> Event {
        let arg = Argument {
            name: "msg".to_string(),
            ty: CLType::String,
            is_ref: false,
            is_slice: false,
            is_required: true
        };
        Event {
            name: "Info".to_string(),
            args: vec![arg]
        }
    }
}
