//! `IntoRuntimeArgs` reads named fields; an enum or a tuple struct has none.
use odra::IntoRuntimeArgs;

#[derive(IntoRuntimeArgs)]
pub enum Args {
    A,
    B
}

#[derive(IntoRuntimeArgs)]
pub struct Tuple(u32, u32);

fn main() {}
