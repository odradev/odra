use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=WASM_CLIENT_SK");
    match env::var("WASM_CLIENT_SK") {
        Ok(secret_key) => println!("cargo:rustc-env=WASM_CLIENT_SK={}", secret_key),
        Err(_) => {
            println!("cargo:rustc-env=WASM_CLIENT_SK=");
            println!(
                "cargo:warning=WASM_CLIENT_SK is not set, the key is empty and the client will not be able to sign"
            );
        }
    }
}
