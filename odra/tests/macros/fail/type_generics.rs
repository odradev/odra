//! An Odra type, an error enum and an event are one concrete type each.
use odra::prelude::*;

#[odra::odra_type]
pub struct GenericType<T> {
    pub value: T
}

#[odra::odra_type]
pub enum GenericEnum<T> {
    Value(T)
}

#[odra::odra_error]
pub enum GenericError<'a> {
    Fail(&'a u32)
}

#[odra::event]
pub struct GenericEvent<T> {
    pub value: T
}

#[odra::event]
pub struct BorrowedEvent<'a> {
    pub value: &'a u32
}

fn main() {}
