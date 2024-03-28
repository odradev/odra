pub fn build() {
    flags().iter().for_each(|flag| println!("{}", flag));
}

fn flags() -> Vec<String> {
    let mut flags = vec![];
    flags.push("cargo:rerun-if-env-changed=ODRA_MODULE".to_string());
    let module = std::env::var("ODRA_MODULE").unwrap_or_else(|_| "".to_string());
    let msg = format!("cargo:rustc-cfg=odra_module=\"{}\"", module);
    flags.push(msg);
    flags
}

#[cfg(test)]
mod test {
    #[test]
    fn test_flags() {
        std::env::remove_var("ODRA_MODULE");
        let flags = super::flags();
        assert_eq!(flags.len(), 2);
        assert_eq!(flags[0], "cargo:rerun-if-env-changed=ODRA_MODULE");
        assert_eq!(flags[1], "cargo:rustc-cfg=odra_module=\"\"");

        std::env::set_var("ODRA_MODULE", "test");
        let flags = super::flags();
        assert_eq!(flags.len(), 2);
        assert_eq!(flags[0], "cargo:rerun-if-env-changed=ODRA_MODULE");
        assert_eq!(flags[1], "cargo:rustc-cfg=odra_module=\"test\"");
    }
}
