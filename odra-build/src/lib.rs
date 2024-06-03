use serde::Serialize;

pub fn build() {
    flags().iter().for_each(|flag| println!("{}", flag));
}

pub fn schema<B, S>(legacy_schema: B, schema: S)
where
    B: Serialize,
    S: Serialize
{
    let module = std::env::var("ODRA_MODULE").expect("ODRA_MODULE environment variable is not set");
    let module = to_snake_case(&module);

    write_schema_file("resources/casper_contract_schemas", &module, schema);

    write_schema_file("resources/legacy", &module, legacy_schema);
}

fn write_schema_file<T>(path: &str, module: &str, schema: T)
where
    T: Serialize
{
    let json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema to JSON");
    if !std::path::Path::new(path).exists() {
        std::fs::create_dir_all(path).expect("Failed to create resources directory");
    }
    let filename = format!("{}/{}_schema.json", path, module);
    let mut schema_file = std::fs::File::create(filename).expect("Failed to create schema file");

    std::io::Write::write_all(&mut schema_file, &json.into_bytes())
        .expect("Failed to write to schema file");
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut is_first = true;

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !is_first {
                if let Some(next) = chars.peek() {
                    if next.is_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
        is_first = false;
    }

    result
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
