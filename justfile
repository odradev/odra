default:
    just --list

test-examples-on-mockvm:
    cargo test --manifest-path examples/Cargo.toml --target-dir target

test-odra:
    cargo test