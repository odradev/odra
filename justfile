CARGO_ODRA_GIT_REPO := "https://github.com/odradev/cargo-odra"
CARGO_ODRA_BRANCH := "release/0.1.3"
BINARYEN_VERSION := "version_116"
BINARYEN_CHECKSUM := "c55b74f3109cdae97490faf089b0286d3bba926bb6ea5ed00c8c784fc53718fd"

default:
    just --list

clippy:
    cargo clippy --all-targets -- -D warnings
    cd odra-casper/proxy-caller && cargo clippy --target=wasm32-unknown-unknown -- -D warnings -A clippy::single-component-path-imports
    cd examples && cargo clippy --all-targets -- -D warnings 
    cd modules && cargo clippy --all-targets -- -D warnings
    cd benchmark && cargo clippy --all-targets -- -D warnings

lint: clippy
    cargo fmt
    cd odra-casper/proxy-caller && cargo fmt
    cd examples && cargo fmt
    cd modules && cargo fmt
    cd benchmark && cargo fmt

check-lint: clippy
    cargo fmt -- --check
    cd odra-casper/proxy-caller && cargo fmt -- --check
    cd modules && cargo fmt -- --check
    cd modules && cargo check --all-targets
    cd examples && cargo fmt -- --check
    cd examples && cargo check --all-targets
    cd benchmark && cargo fmt -- --check
    cd benchmark && cargo check --all-targets --features=benchmark

install-cargo-odra:
    rustup toolchain install stable
    cargo +stable install cargo-odra --git {{CARGO_ODRA_GIT_REPO}} --branch {{CARGO_ODRA_BRANCH}} --locked

prepare-test-env: install-cargo-odra
    rustup target add wasm32-unknown-unknown
    rustup component add llvm-tools-preview
    cargo +stable install grcov
    sudo apt install wabt
    wget https://github.com/WebAssembly/binaryen/releases/download/{{BINARYEN_VERSION}}/binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz || { echo "Download failed"; exit 1; }
    sha256sum binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz | grep {{BINARYEN_CHECKSUM}} || { echo "Checksum verification failed"; exit 1; }
    tar -xzf binaryen-{{BINARYEN_VERSION}}-x86_64-linux.tar.gz || { echo "Extraction failed"; exit 1; }
    sudo cp binaryen-{{BINARYEN_VERSION}}/bin/wasm-opt /usr/local/bin/wasm-opt

start-casper-node:
    docker run --rm -it --name mynctl -d -p 11101:11101 -p 14101:14101 -p 18101:18101 makesoftware/casper-nctl

build-proxy-callers:
    cd odra-casper/proxy-caller && cargo build --release --target wasm32-unknown-unknown --target-dir ../../target
    wasm-strip target/wasm32-unknown-unknown/release/proxy_caller.wasm
    wasm-strip target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm
    wasm-opt --signext-lowering target/wasm32-unknown-unknown/release/proxy_caller.wasm  -o target/wasm32-unknown-unknown/release/proxy_caller.wasm
    wasm-opt --signext-lowering target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm  -o target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm
    cp target/wasm32-unknown-unknown/release/proxy_caller.wasm \
        odra-casper/test-vm/resources/proxy_caller.wasm
    cp target/wasm32-unknown-unknown/release/proxy_caller_with_return.wasm \
        odra-casper/test-vm/resources/proxy_caller_with_return.wasm

test-odra:
    cargo test

test-examples-on-odravm:
    cd examples && cargo odra test
    cd examples/ourcoin && cargo odra test

test-examples-on-casper:
    cd examples && cargo odra test -b casper
    cd examples/ourcoin && cargo odra test -b casper

test-examples: test-examples-on-odravm test-examples-on-casper

test-modules-on-odravm:
    cd modules && cargo odra test

test-modules-on-casper:
    cd modules && cargo odra test -b casper

test-modules: test-modules-on-odravm test-modules-on-casper

test: test-odra test-examples test-modules

test-template name:
    cd tests && cargo odra new -n {{name}} --template {{name}} -s ../ \
        && cd {{name}} \
        && cargo odra test \
        && cargo odra test -b casper \
        && cargo odra schema

test-templates:
    rm -rf tests
    mkdir -p tests
    just test-template cep18
    just test-template full
    just test-template workspace
    just test-template cep78

run-nctl:
    docker run --rm -it --name mynctl -d -p 11101:11101 -p 14101:14101 -p 18101:18101 makesoftware/casper-nctl:v155

test-livenet:
    set shell := bash
    mkdir -p examples/.node-keys
    cp modules/wasm/Erc20.wasm examples/wasm/
    # Extract the secret keys from the local Casper node
    docker exec mynctl /bin/bash -c "cat /home/casper/casper-node/utils/nctl/assets/net-1/users/user-1/secret_key.pem" > examples/.node-keys/secret_key.pem
    docker exec mynctl /bin/bash -c "cat /home/casper/casper-node/utils/nctl/assets/net-1/users/user-2/secret_key.pem" > examples/.node-keys/secret_key_1.pem
    # Run the tests
    cd examples && ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.node-keys/secret_key.pem ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101 ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1 ODRA_CASPER_LIVENET_KEY_1=.node-keys/secret_key_1.pem  cargo run --bin livenet_tests --features=livenet
    rm -rf examples/.node-keys

run-example-erc20-on-livenet:
    cd examples && cargo run --bin erc20-on-livenet --features casper-livenet --no-default-features

clean:
    cargo clean
    cd examples && cargo odra clean
    cd modules && cargo odra clean
    rm -f Cargo.lock
    cd examples && rm -f Cargo.lock
    cd modules && rm -f Cargo.lock

coverage:
    rm -rf target/coverage/
    mkdir -p target/coverage/
    CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/coverage/cargo-test-%p-%m.profraw' cargo test
    cd examples && CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='../target/coverage/cargo-test-%p-%m.profraw' cargo test
    cd modules && CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='../target/coverage/cargo-test-%p-%m.profraw' cargo test
    # Uncomment the following line to generate local HTML report
    # grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore '../*' --ignore "/*" -o target/coverage/html
    grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/tests.lcov

benchmark:
    cd benchmark && cargo odra build && ODRA_BACKEND=casper cargo run --bin benchmark --features=benchmark

evaluate-benchmark: benchmark
    cd benchmark && cargo run --bin evaluate_benchmark gas_report.json base/gas_report.json

run-client:
    cd modules/client && wasm-pack build
    cd modules/client/www && npm run start