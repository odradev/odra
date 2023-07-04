use std::env;

pub fn build() {
    println!("cargo:rerun-if-env-changed=ODRA_BACKEND");
    let backend = env::var("ODRA_BACKEND").unwrap_or_else(|_| "mock-vm".to_string());
    let msg = format!("cargo:rustc-cfg=odra_backend=\"{}\"", backend);
    println!("{}", msg);
}
