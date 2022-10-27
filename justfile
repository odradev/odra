default:
    just --list

test-examples-on-mockvm:
    cargo test -p odra-examples

test-odra:
    cargo test