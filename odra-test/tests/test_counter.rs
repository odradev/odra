use cucumber::{given, then, when, World};
use odra_test::bdd::SyncRunner;

mod counter {
    use std::{cell::RefCell, thread_local};

    thread_local! {
        static NUMBER: RefCell<u32> = RefCell::new(0);
    }

    pub fn increment() {
        NUMBER.with(|n| {
            *n.borrow_mut() += 1;
        });
    }

    pub fn value() -> u32 {
        NUMBER.with(|n| n.borrow().clone())
    }
}

#[derive(Debug, Clone, Default, World)]
pub struct MyWorld;

#[given("empty counter")]
fn empty_counter(_world: &mut MyWorld) {
    assert_eq!(counter::value(), 0);
}

#[when(expr = "counter is incremented by {int}")]
fn increment_counter(_world: &mut MyWorld, increment: u32) {
    for _ in 0..increment {
        counter::increment();
    }
}

#[then(expr = "counter is {int}")]
fn counter_is(_world: &mut MyWorld, expected: u32) {
    assert_eq!(counter::value(), expected);
}

fn main() {
    odra_test::bdd::run_sync::<MyWorld, _>("tests/features/");
}
