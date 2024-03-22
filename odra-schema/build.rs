//! This build script is used to set the `RUSTC_VERSION` environment variable to the current Rust
use std::process::Command;

fn main() {
    // Get the Rust version
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Failed to get Rust version");

    let rust_version =
        String::from_utf8(output.stdout).expect("Failed to convert Rust version to string");

    // Set the CARGO_PKG_RUST_VERSION variable
    println!("cargo:rustc-env=RUSTC_VERSION={}", rust_version);
}
