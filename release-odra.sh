# Before running, comment out dev-dependencies in odra-proc-macros crate.

cargo publish -p odra-types && sleep 100
cargo publish -p odra-utils && sleep 100
cargo publish -p odra-mock-vm-types && sleep 100
cargo publish -p odra-mock-vm && sleep 100
cargo publish -p odra-ir && sleep 100
cargo publish -p odra-codegen && sleep 100
cargo publish -p odra-proc-macros --allow-dirty && sleep 100
cargo publish -p odra-casper-shared && sleep 100
cargo publish -p odra-casper-types && sleep 100
cargo publish -p odra-casper-test-env && sleep 100
cargo publish -p odra-casper-codegen && sleep 100
cargo publish -p odra-casper-backend && sleep 100
cargo publish -p odra && sleep 100
cd modules && cargo publish
