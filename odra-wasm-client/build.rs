use std::env;

fn main() {
    if let Ok(secret_key) = env::var("WASM_CLIENT_SK") {
        println!("cargo:rustc-env=WASM_CLIENT_SK={}", secret_key);
    } else {
        panic!("WASM_CLIENT_SK environment variable must be set at build time");
    }
}
