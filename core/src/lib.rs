mod contract_def;
mod instance;
mod event;
mod mapping;
mod variable;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_test_env::TestEnv;
        pub use odra_mock_env::Env;
    } else if #[cfg(feature = "wasm")] {
        mod external_api;
        pub use external_api::env::Env;
        pub use external_api::test_env::TestEnv;
    } else if #[cfg(feature = "wasm-test")] {
        mod external_api;
        pub use external_api::Env;
        pub use external_api::test_env::TestEnv;
    } else {
        compile_error!("Unsupported feature");
    }
}
