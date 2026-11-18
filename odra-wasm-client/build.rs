use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=WASM_CLIENT_SK");
    match env::var("WASM_CLIENT_SK") {
        // A cargo directive ends at the first line break, and a PEM file has several lines: pass
        // the key on one line. `SecretKey::from_pem` reads it either way.
        Ok(secret_key) => println!(
            "cargo:rustc-env=WASM_CLIENT_SK={}",
            secret_key.replace(['\r', '\n'], "")
        ),
        Err(_) => {
            println!("cargo:rustc-env=WASM_CLIENT_SK=");
            println!(
                "cargo:warning=WASM_CLIENT_SK is not set, the key is empty and the client will not be able to sign"
            );
        }
    }
}
