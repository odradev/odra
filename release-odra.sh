# Before running, comment out dev-dependencies in odra-proc-macros crate.

cargo publish -p odra-types
cargo publish -p odra-utils
cargo publish -p odra-mock-vm-types
cargo publish -p odra-mock-vm
cargo publish -p odra-ir
cargo publish -p odra-codegen
cargo publish -p odra-proc-macros --allow-dirty
cargo publish -p odra-casper-types
cargo publish -p odra-casper-shared
cargo publish -p odra-casper-test-env
cargo publish -p odra-casper-codegen
cargo publish -p odra-casper-backend
cargo publish -p odra-casper-livenet
cargo publish -p odra
cd modules && cargo publish
