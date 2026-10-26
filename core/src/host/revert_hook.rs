//! The panic that unwinds a reverted contract call on a host backend, and the process-wide panic
//! hook that reports it as a revert instead of a crash.
//!
//! A backend that runs contract code natively (OdraVM, the getters of livenet) stops a reverted
//! call by panicking and catches the panic at the call boundary. Rust's panic hook would print it
//! like any crash, so the backend marks the revert first ([mark_revert]), and the hook installed by
//! [install] hands a marked panic to the backend's reporter and every other panic to the hook that
//! was installed before.

use std::boxed::Box;
use std::cell::Cell;
use std::panic::PanicHookInfo;
use std::sync::Once;

/// Reports a marked revert, see [mark_revert]. Returns `false` to leave the panic to the hook
/// installed before, like any other panic.
pub type RevertReporter = fn(&PanicHookInfo<'_>) -> bool;

std::thread_local! {
    static REVERT: Cell<Option<RevertReporter>> = const { Cell::new(None) };
}

/// Marks the next panic on this thread as a revert, reported by `report`. A backend calls it right
/// before it panics to unwind a reverted call; that panic consumes the mark.
pub fn mark_revert(report: RevertReporter) {
    REVERT.with(|revert| revert.set(Some(report)));
}

/// Installs the revert panic hook, once per process.
///
/// The hook keeps the one installed before and passes it every panic that is not a marked revert,
/// assertions included, so those are reported as usual (the default hook prints the message, the
/// location and the backtrace).
pub fn install() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            match REVERT.with(|revert| revert.take()) {
                Some(report) if report(panic_info) => {}
                _ => previous_hook(panic_info)
            }
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    std::thread_local! {
        static REPORTED: Cell<u32> = const { Cell::new(0) };
    }

    fn count(_: &PanicHookInfo<'_>) -> bool {
        REPORTED.with(|reported| reported.set(reported.get() + 1));
        true
    }

    #[test]
    fn a_marked_panic_goes_to_its_reporter_and_consumes_the_mark() {
        install();
        mark_revert(count);
        assert!(catch_unwind(|| panic!("Revert: marked")).is_err());
        assert_eq!(REPORTED.with(Cell::get), 1);
        // An ordinary panic goes to the previous hook.
        assert!(catch_unwind(|| panic!("not a revert")).is_err());
        assert_eq!(REPORTED.with(Cell::get), 1);
    }
}
