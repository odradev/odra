default:
    just --list

clippy:
    # TODO: add --all-targets to cargo clippy when tests are fixed
    cargo clippy -- -D warnings
    cd odra-casper/proxy-caller && cargo clippy --target=wasm32-unknown-unknown -- -D warnings -A clippy::single-component-path-imports
    # TODO: uncomment when examples are rewritten
    # cd examples && cargo clippy -- -D warnings -A clippy::assign-op-pattern
    # TODO: uncomment when modules are rewritten
    # cd modules && cargo clippy -- -D warnings

lint: clippy
    cargo fmt
    cd odra-casper/proxy-caller && cargo fmt
    cd examples && cargo fmt
    cd modules && cargo fmt

check-lint: clippy
    cargo fmt -- --check
    cd odra-casper/proxy-caller && cargo fmt -- --check
    cd examples2 && cargo fmt -- --check
#    cd examples && cargo fmt -- --check
#    cd modules && cargo fmt -- --check
#    cd examples && cargo check --no-default-features -F casper-livenet

install-cargo-odra:
    cargo install cargo-odra --locked

prepare-test-env: install-cargo-odra
    rustup target add wasm32-unknown-unknown
    sudo apt install wabt

build-proxy-callers:
    cd odra-casper/proxy-caller && cargo build --release --target wasm32-unknown-unknown --target-dir ../../target
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

build-erc20:
    cd examples2 && ODRA_MODULE=Erc20 cargo build --release --target wasm32-unknown-unknown --bin build_contract
    wasm-strip examples2/target/wasm32-unknown-unknown/release/build_contract.wasm
    rm -rf examples2/wasm/Erc20.wasm
    mkdir -p examples2/wasm
    mv examples2/target/wasm32-unknown-unknown/release/build_contract.wasm examples2/wasm/Erc20.wasm

test-erc20: build-erc20
    cd examples2 && cargo test --lib erc20 -- --nocapture
    cd examples2 && ODRA_BACKEND=casper cargo test --lib erc20 -- --nocapture

build-counter-pack:
    cd examples2 && ODRA_MODULE=CounterPack cargo build --release --target wasm32-unknown-unknown --bin build_contract
    wasm-strip examples2/target/wasm32-unknown-unknown/release/build_contract.wasm
    cp examples2/target/wasm32-unknown-unknown/release/build_contract.wasm examples2/wasm/CounterPack.wasm

test-counter-pack: build-counter-pack
    cd examples2 && ODRA_BACKEND=casper cargo test --lib counter_pack -- --nocapture

build-erc20-schema:
    cd examples2 && ODRA_MODULE=Erc20 cargo run --bin build_schema
