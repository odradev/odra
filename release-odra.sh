# Before running, comment out dev-dependencies in odra-proc-macros crate.

# Exit immediately if a command exits with a non-zero status.
set -e

cargo publish -p odra-build
cargo publish -p odra-core
cargo publish -p odra-macros
cargo publish -p odra-vm
cargo publish -p odra-casper-wasm-env 
cargo publish -p odra-schema
cargo publish -p odra
cargo publish -p odra-casper-test-vm
cargo publish -p odra-test
cargo publish -p odra-casper-rpc-client
cargo publish -p odra-casper-livenet-env
cd modules && cargo publish --allow-dirty --no-verify
