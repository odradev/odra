//! Each `#[odra::module(..)]` argument takes a value of one shape.
use odra::prelude::*;

#[odra::event]
pub struct Transfer {
    pub amount: u32
}

#[odra::odra_error]
pub enum Error {
    Fail = 1
}

#[odra::module(events = Transfer)]
pub struct NotAList {
    value: Var<u32>
}

#[odra::module(events(Transfer))]
pub struct NoEquals {
    value: Var<u32>
}

#[odra::module(name = Token)]
pub struct NameNotAString {
    value: Var<u32>
}

#[odra::module(version = 1)]
pub struct VersionNotAString {
    value: Var<u32>
}

#[odra::module(errors = "Error")]
pub struct ErrorsAString {
    value: Var<u32>
}

#[odra::module(name)]
pub struct NoValue {
    value: Var<u32>
}

#[odra::module(events = [Transfer,, ])]
pub struct ListWithAHole {
    value: Var<u32>
}

#[odra::module(factory = "on")]
pub struct FactoryAString {
    value: Var<u32>
}

fn main() {}
