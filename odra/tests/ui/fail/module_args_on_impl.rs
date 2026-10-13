//! Since 2.10 the module configuration belongs on the struct only (`factory` excepted).
use odra::prelude::*;

#[odra::event]
pub struct Event {
    pub value: u32
}

#[odra::odra_error]
pub enum Error {
    Fail = 1
}

#[odra::module]
pub struct WithEvents {
    value: Var<u32>
}

#[odra::module(events = [Event])]
impl WithEvents {}

#[odra::module]
pub struct WithErrors {
    value: Var<u32>
}

#[odra::module(errors = Error)]
impl WithErrors {}

#[odra::module]
pub struct WithName {
    value: Var<u32>
}

#[odra::module(factory = on, name = "Name")]
impl WithName {}

#[odra::module]
pub struct WithVersion {
    value: Var<u32>
}

#[odra::module(version = "1.0.0")]
impl WithVersion {}

fn main() {}
