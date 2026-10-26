//! The Odra macros from the user's side: every file in `macros/pass` is built as a crate against
//! `odra` and must compile without warnings, every file in `macros/fail` must fail with exactly
//! the diagnostics in its `.stderr` file (message and span). The macro crate's own tests check
//! what is generated; these check that it compiles and what a mistake looks like.
//!
//! The `.stderr` snapshots depend on the compiler version: regenerate them with the toolchain
//! pinned in `rust-toolchain` (`TRYBUILD=overwrite cargo test -p odra --test macros`) and review
//! the diff.

#[test]
fn macros() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macros/pass/*.rs");
    t.compile_fail("tests/macros/fail/*.rs");
}
