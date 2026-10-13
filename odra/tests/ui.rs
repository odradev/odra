//! Compile-pass and compile-fail tests of the Odra macros.
//!
//! The `.stderr` snapshots depend on the compiler version: regenerate them with the toolchain
//! pinned in `rust-toolchain` (`TRYBUILD=overwrite cargo test -p odra --test ui`) and review the
//! diff.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
}
