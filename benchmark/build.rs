use std::env;

pub fn main() {
    println!("cargo::rustc-check-cfg=cfg(odra_module, values(any()))");
    println!("cargo:rerun-if-env-changed=ODRA_MODULE");
    let module = env::var("ODRA_MODULE").unwrap_or_else(|_| "".to_string());
    let msg = format!("cargo:rustc-cfg=odra_module=\"{}\"", module);
    println!("{}", msg);
}
