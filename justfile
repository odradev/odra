default:
    just --list

test-examples-on-mockvm:
    cargo test -p odra-examples

test-odra:
    cargo test

lint:
    cargo fmt
    cargo clippy --all-targets -- -D warnings

test-modules-on-casper:
    cd modules && cargo odra test -b casper