default:
    just --list

clippy:
    cargo clippy --all-targets -- -D warnings 
    cd odra-casper/proxy-caller && cargo clippy --target=wasm32-unknown-unknown -- -D warnings -A clippy::single-component-path-imports
    cd examples && cargo clippy --all-targets -- -D warnings -A clippy::assign-op-pattern
    cd modules && cargo clippy --all-targets -- -D warnings

lint: clippy
    cargo fmt
    cd odra-casper/proxy-caller && cargo fmt
    cd examples && cargo fmt
    cd modules && cargo fmt

check-lint: clippy
    cargo fmt -- --check
    cd odra-casper/proxy-caller && cargo fmt -- --check 
    cd examples && cargo fmt -- --check
    cd modules && cargo fmt -- --check
    cd examples && cargo check --no-default-features -F casper-livenet

install-cargo-odra:
    cargo install cargo-odra --locked

prepare-test-env: install-cargo-odra
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

build-proxy-callers:
    cargo build -p odra-casper-proxy-caller --release --target wasm32-unknown-unknown
    wasm-strip target/wasm32-unknown-unknown/release/proxy_caller.wasm
    wasm-strip target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm
    cp target/wasm32-unknown-unknown/release/proxy_caller.wasm \
        odra-casper/livenet/resources/proxy_caller.wasm
    cp target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm \
        odra-casper/test-env/resources/proxy_caller_with_return.wasm

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

run-example-erc20-on-livenet:
    cd examples && cargo run --bin erc20-on-livenet --features casper-livenet --no-default-features

clean:
    cargo clean
    cd examples && cargo odra clean
    cd modules && cargo odra clean
    rm -f Cargo.lock
    cd examples && rm -f Cargo.lock
    cd modules && rm -f Cargo.lock
