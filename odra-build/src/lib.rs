use serde::Serialize;
use std::path::Path;

pub fn build() {
    // Declare the cfg flags set below, so rustc's check-cfg does not report them as unexpected.
    println!("cargo::rustc-check-cfg=cfg(odra_module, values(any()))");
    // Allow the `odra_contract_schema` cfg flag to be set.
    println!("cargo::rustc-check-cfg=cfg(odra_contract_schema)");
    println!("cargo::rustc-check-cfg=cfg(odra_backend, values(any()))");
    // Load flags.
    flags().iter().for_each(|flag| println!("{}", flag));
    contract_schema_flags()
        .iter()
        .for_each(|flag| println!("{}", flag));
}

const CONTRACT_SCHEMAS_DIR: &str = "resources/casper_contract_schemas";

/// Writes the contract schema of the module named by `ODRA_MODULE` to
/// `resources/casper_contract_schemas/<module>_schema.json`.
pub fn contract_schema<S: Serialize>(schema: S) {
    let module = std::env::var("ODRA_MODULE").expect("ODRA_MODULE environment variable is not set");
    let module = to_snake_case(module_file_name(&module));

    write_schema_file(CONTRACT_SCHEMAS_DIR, &module, schema);
}

/// Kept for the `bin/build_schema.rs` of projects generated before [`contract_schema`]: the legacy
/// schema is no longer written, only the contract schema.
#[doc(hidden)]
pub fn schema<B, S>(_legacy_schema: B, schema: S)
where
    B: Serialize,
    S: Serialize
{
    contract_schema(schema);
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

/// The struct part of an `ODRA_MODULE` value.
///
/// `cargo odra` passes a crate-qualified module name (`odra_modules::Erc20`), while a manual
/// build may pass the bare struct name (`Erc20`); both name the same schema file.
fn module_file_name(module: &str) -> &str {
    module.rsplit("::").next().unwrap_or(module)
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

/// When building a wasm contract, looks for the module's contract schema file
/// (`resources/casper_contract_schemas/<module>_schema.json`) in the crate directory and its
/// ancestors (to support workspaces). If found, sets the `odra_contract_schema` cfg flag and
/// exposes the path of a minified copy via `ODRA_CONTRACT_SCHEMA_PATH`, so the schema is
/// embedded in the wasm and stored under the `__contract_schema` named key on install.
///
/// The file in `resources` stays pretty-printed for people and diffs; the embedded copy is
/// written to `OUT_DIR` without whitespace, so the wasm does not carry the indentation.
fn contract_schema_flags() -> Vec<String> {
    let mut flags = vec![];
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return flags;
    }
    let module = match std::env::var("ODRA_MODULE") {
        Ok(module) if !module.is_empty() => module,
        _ => return flags
    };
    let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return flags;
    };
    let file_name = format!("{}_schema.json", to_snake_case(&module));
    let schema_path = Path::new(&manifest_dir)
        .ancestors()
        .map(|dir| dir.join(CONTRACT_SCHEMAS_DIR).join(&file_name))
        .find(|path| path.is_file());

    match schema_path {
        Some(path) => {
            flags.push(format!("cargo:rerun-if-changed={}", path.display()));
            flags.push("cargo:rustc-cfg=odra_contract_schema".to_string());
            let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
            let embedded = match minified_copy(&path, Path::new(&out_dir)) {
                Ok(minified) => minified,
                Err(err) => {
                    flags.push(format!(
                        "cargo:warning=Contract schema {} could not be minified ({}); embedding it as is.",
                        path.display(),
                        err
                    ));
                    path
                }
            };
            flags.push(format!(
                "cargo:rustc-env=ODRA_CONTRACT_SCHEMA_PATH={}",
                embedded.display()
            ));
        }
        None => {
            // Rerun once the schema gets generated.
            let path = Path::new(&manifest_dir)
                .join(CONTRACT_SCHEMAS_DIR)
                .join(&file_name);
            flags.push(format!("cargo:rerun-if-changed={}", path.display()));
            // Dependencies are built with the same `ODRA_MODULE`, so warn only in the crate that
            // defines the contract.
            if !odra_toml_lists_contract(Path::new(&manifest_dir), &module) {
                return flags;
            }
            flags.push(format!(
                "cargo:warning=Contract schema for {} not found at {}. The contract will be built without an embedded schema. Run `cargo odra schema -c {}` to generate it.",
                module,
                path.display(),
                module
            ));
        }
    }
    flags
}

/// Writes the JSON at `path` to `out_dir` under the same file name, without the whitespace
/// between tokens, and returns the path of the copy.
fn minified_copy(path: &Path, out_dir: &Path) -> Result<std::path::PathBuf, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<serde_json::Value>(&content).map_err(|e| e.to_string())?;
    let file_name = path.file_name().ok_or("the schema path has no file name")?;
    let copy = out_dir.join(file_name);
    std::fs::write(&copy, minify_json(&content)).map_err(|e| e.to_string())?;
    Ok(copy)
}

/// Drops the whitespace outside string literals of a valid JSON document, keeping the keys in
/// their order.
fn minify_json(json: &str) -> String {
    let mut out = String::with_capacity(json.len());
    let mut in_string = false;
    let mut escaped = false;
    for c in json.chars() {
        if in_string {
            out.push(c);
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if !c.is_whitespace() {
            out.push(c);
        }
    }
    out
}

/// Checks if the `Odra.toml` closest to `dir` lists a contract named `contract_name`
/// (the last segment of its `fqn`).
fn odra_toml_lists_contract(dir: &Path, contract_name: &str) -> bool {
    let Some(odra_toml) = dir
        .ancestors()
        .map(|dir| dir.join("Odra.toml"))
        .find(|path| path.is_file())
    else {
        return false;
    };
    let Ok(content) = std::fs::read_to_string(odra_toml) else {
        return false;
    };
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("fqn"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|fqn| fqn.trim().trim_matches('"'))
        .any(|fqn| fqn.rsplit("::").next() == Some(contract_name))
}

fn flags() -> Vec<String> {
    let mut flags = vec![];
    flags.push("cargo:rerun-if-env-changed=ODRA_MODULE".to_string());
    let module = std::env::var("ODRA_MODULE").unwrap_or_else(|_| "".to_string());
    let msg = format!("cargo:rustc-cfg=odra_module=\"{}\"", module);
    flags.push(msg);

    flags.push("cargo:rerun-if-env-changed=ODRA_BACKEND".to_string());
    let backend_env = std::env::var("ODRA_BACKEND").unwrap_or_else(|_| "".to_string());
    let msg = format!("cargo:rustc-cfg=odra_backend=\"{}\"", backend_env);
    flags.push(msg);

    flags
}

#[cfg(test)]
mod test {
    #[test]
    fn test_odra_toml_lists_contract() {
        let root = std::env::temp_dir().join(format!("odra-build-test-{}", std::process::id()));
        let member = root.join("member");
        std::fs::create_dir_all(&member).unwrap();
        std::fs::write(
            root.join("Odra.toml"),
            "[[contracts]]\nfqn = \"features::storage::variable::DogContract\"\n\n[[contracts]]\nfqn=\"Counter\"\n"
        )
        .unwrap();

        // Found in the crate directory and in its ancestors.
        assert!(super::odra_toml_lists_contract(&root, "DogContract"));
        assert!(super::odra_toml_lists_contract(&member, "DogContract"));
        assert!(super::odra_toml_lists_contract(&root, "Counter"));
        // Only the last segment of the fqn matches.
        assert!(!super::odra_toml_lists_contract(&root, "variable"));
        assert!(!super::odra_toml_lists_contract(&root, "Dog"));

        std::fs::remove_dir_all(&root).unwrap();
        // No Odra.toml at all.
        assert!(!super::odra_toml_lists_contract(&root, "DogContract"));
    }

    #[test]
    fn test_minified_copy() {
        let root = std::env::temp_dir().join(format!("odra-build-minify-{}", std::process::id()));
        let out_dir = root.join("out");
        std::fs::create_dir_all(&out_dir).unwrap();
        let schema = root.join("counter_schema.json");
        std::fs::write(
            &schema,
            "{\n  \"name\": \"Counter\",\n  \"errors\": [ ]\n}\n"
        )
        .unwrap();

        let copy = super::minified_copy(&schema, &out_dir).unwrap();
        assert_eq!(copy, out_dir.join("counter_schema.json"));
        assert_eq!(
            std::fs::read_to_string(copy).unwrap(),
            "{\"name\":\"Counter\",\"errors\":[]}"
        );
        // Whitespace and escaped quotes inside strings are kept.
        assert_eq!(
            super::minify_json("{ \"a\" : \"x y \\\" z\" ,\n \"b\": 1 }"),
            "{\"a\":\"x y \\\" z\",\"b\":1}"
        );

        std::fs::write(&schema, "not json").unwrap();
        assert!(super::minified_copy(&schema, &out_dir).is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_flags() {
        std::env::remove_var("ODRA_MODULE");
        std::env::remove_var("ODRA_BACKEND");
        let flags = super::flags();
        assert_eq!(flags.len(), 4);
        assert_eq!(flags[0], "cargo:rerun-if-env-changed=ODRA_MODULE");
        assert_eq!(flags[1], "cargo:rustc-cfg=odra_module=\"\"");
        assert_eq!(flags[2], "cargo:rerun-if-env-changed=ODRA_BACKEND");
        assert_eq!(flags[3], "cargo:rustc-cfg=odra_backend=\"\"");

        std::env::set_var("ODRA_MODULE", "test");
        std::env::set_var("ODRA_BACKEND", "backend_test");
        let flags = super::flags();
        assert_eq!(flags.len(), 4);
        assert_eq!(flags[0], "cargo:rerun-if-env-changed=ODRA_MODULE");
        assert_eq!(flags[1], "cargo:rustc-cfg=odra_module=\"test\"");
        assert_eq!(flags[2], "cargo:rerun-if-env-changed=ODRA_BACKEND");
        assert_eq!(flags[3], "cargo:rustc-cfg=odra_backend=\"backend_test\"");
    }

    #[test]
    fn test_schema_file_name() {
        let name = |module| super::to_snake_case(super::module_file_name(module));
        assert_eq!(name("Erc20"), "erc20");
        assert_eq!(name("odra_modules::Erc20"), "erc20");
        assert_eq!(name("odra_modules::Cep18"), "cep18");
        assert_eq!(name("odra_examples::PartyContract"), "party_contract");
    }
}
