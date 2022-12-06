CARGO_ODRA_GIT := "https://github.com/odradev/cargo-odra"
CARGO_ODRA_BRANCH := "0.0.4"

default:
    just --list

clippy:
    cargo clippy --all-targets -- -D warnings
    cd examples && cargo clippy --all-targets -- -D warnings -A clippy::assign-op-pattern
    cd modules && cargo clippy --all-targets -- -D warnings

lint: clippy
    cargo fmt
    cd examples && cargo fmt
    cd modules && cargo fmt

check-lint: clippy
    cargo fmt -- --check
    cd examples && cargo fmt -- --check
    cd modules && cargo fmt -- --check

install-cargo-odra:
    cargo install --git {{CARGO_ODRA_GIT}} --branch {{CARGO_ODRA_BRANCH}}

prepare-test-env: install-cargo-odra
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

test-odra:
    cargo test

test-examples-on-mockvm:
    cd examples && cargo odra test

test-examples-on-casper:
    cd examples && cargo odra test -b casper

test-examples: test-examples-on-mockvm test-examples-on-casper

test-modules-on-mockvm:
    cd modules && cargo odra test

test-modules-on-casper:
    cd modules && cargo odra test -b casper

test-modules: test-modules-on-mockvm test-modules-on-casper

test: test-odra test-examples test-modules

run-examples-token-name-reader:
    cd examples && cargo run --no-default-features -F casper-livenet
