//! A revert of a locally executed call unwinds with a panic, which the host catches and returns
//! as an error. The hook keeps such panics quiet, the revert is logged instead.
use std::cell::Cell;
use std::sync::Once;

thread_local! {
    static REVERTING: Cell<bool> = const { Cell::new(false) };
}

/// Marks the next panic on this thread as an Odra revert. Called right before the contract env
/// panics to unwind a reverted call.
pub(crate) fn mark_revert() {
    REVERTING.with(|r| r.set(true));
}

/// Installs the livenet panic hook, once per process.
///
/// Reverts print nothing. Any other panic goes to the hook that was installed before, so it is
/// reported as usual.
pub(crate) fn set_livenet_panic_hook() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            if !REVERTING.with(|r| r.replace(false)) {
                previous_hook(panic_info);
            }
        }));
    });
}
