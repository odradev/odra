//! Behavior driven development test utilities for the Odra contracts.

pub mod param;
mod sync_runner;
mod world;
pub use world::OdraWorld;

/// Run the tests ith the default cucumber runner.
pub fn run<T: std::fmt::Debug + cucumber::codegen::WorldInventory, I: AsRef<std::path::Path>>(
    input: I
) {
    futures::executor::block_on(T::run(input));
}

/// Run the tests with synchronous runner.
pub fn run_sync<
    T: cucumber::World
        + cucumber::codegen::WorldInventory
        + std::fmt::Debug
        + Send
        + Sync
        + Default
        + Clone,
    I: AsRef<std::path::Path>
>(
    input: I
) where
    <T as cucumber::World>::Error: std::fmt::Debug
{
    let future = T::cucumber()
        .with_runner(sync_runner::SyncRunner::default())
        .run_and_exit(input);
    futures::executor::block_on(future);
}
