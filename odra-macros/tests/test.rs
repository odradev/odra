pub mod odra {
    pub use casper_types;
}

use casper_types::{runtime_args, RuntimeArgs};
use odra_macros::IntoRuntimeArgs;

#[derive(IntoRuntimeArgs)]
struct WithArgs {
    a: u32,
    b: u32
}

#[derive(IntoRuntimeArgs)]
struct NoArgs;

#[derive(IntoRuntimeArgs)]
#[is_none]
struct IsNone;

#[test]
fn with_args_works() {
    let result: Option<RuntimeArgs> = WithArgs { a: 1, b: 2 }.into();

    let expected = Some(runtime_args! {
        "a" => 1u32,
        "b" => 2u32
    });
    assert_eq!(result, expected);
}

#[test]
fn no_args_works() {
    let result: Option<RuntimeArgs> = NoArgs.into();

    let expected = Some(RuntimeArgs::new());
    assert_eq!(result, expected);
}

#[test]
fn none_works() {
    let args: Option<RuntimeArgs> = IsNone.into();
    let expected = None;
    assert_eq!(args, expected);
}
