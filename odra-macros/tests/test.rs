pub mod odra {
    pub use casper_types;

    pub mod args {
        pub trait EntrypointArgument {
            fn insert_runtime_arg(self, name: &str, args: &mut casper_types::RuntimeArgs);
        }

        impl EntrypointArgument for u32 {
            fn insert_runtime_arg(self, name: &str, args: &mut casper_types::RuntimeArgs) {
                let _ = args.insert(name, self);
            }
        }
    }
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

#[test]
fn with_args_works() {
    let result: RuntimeArgs = WithArgs { a: 1, b: 2 }.into();

    let expected = runtime_args! {
        "a" => 1u32,
        "b" => 2u32
    };
    assert_eq!(result, expected);
}

#[test]
fn no_args_works() {
    let result: RuntimeArgs = NoArgs.into();

    let expected = RuntimeArgs::new();
    assert_eq!(result, expected);
}
