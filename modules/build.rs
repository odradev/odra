//! Odra's contracts build script.
use std::env;

/// Uses the ENV variable `ODRA_MODULE` to set the `odra_module` cfg flag.
pub fn main() {
    println!("cargo:rerun-if-env-changed=ODRA_MODULE");
    let module = env::var("ODRA_MODULE").unwrap_or_else(|_| "".to_string());
    let msg = format!("cargo:rustc-cfg=odra_module=\"{}\"", module);
    println!("{}", msg);
}
