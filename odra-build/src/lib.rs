use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn build() {
    // Declare the cfg flags set below, so rustc's check-cfg does not report them as unexpected.
    println!("cargo::rustc-check-cfg=cfg(odra_module, values(any()))");
    // Allow the `odra_contract_schema` cfg flag to be set.
    println!("cargo::rustc-check-cfg=cfg(odra_contract_schema)");
    println!("cargo::rustc-check-cfg=cfg(odra_backend, values(any()))");
    // Load flags.
    let selection = ModuleSelection::from_env();
    flags(&selection, std::env::var("ODRA_BACKEND").ok())
        .iter()
        .for_each(|flag| println!("{}", flag));
    contract_schema_flags(selection.own_module())
        .iter()
        .for_each(|flag| println!("{}", flag));
    contract_crates_flags()
        .iter()
        .for_each(|flag| println!("{}", flag));
}

/// Tells `odra-build` that the module to build is passed per crate; set by `cargo odra`.
const BUILD_PROTOCOL_ENV: &str = "ODRA_BUILD_PROTOCOL";
/// The first [`BUILD_PROTOCOL_ENV`] value with a per-crate module variable.
const PER_CRATE_PROTOCOL: u32 = 2;
/// The `odra_module` of a crate that does not define the contract being built.
///
/// It is not a struct name, so it selects no module, and it is not empty, so the crate's
/// `bin/build_schema.rs` (`#[cfg(not(odra_module = ""))]`) still knows a contract is being built.
/// It is the same for every contract, which is the point: the crate is not rebuilt between them.
const OTHER_CRATE_MODULE: &str = "_";

/// Name of the variable that names the module of `crate_name` to build:
/// `ODRA_MODULE__odra_modules=Erc20`.
fn crate_module_env(crate_name: &str) -> String {
    format!("ODRA_MODULE__{}", crate_name.replace('-', "_"))
}

/// The module the crate is built for, and the environment variables that decide it.
///
/// Cargo rebuilds a crate whenever a variable its build script watches changes, even when the
/// script prints the same flags again. Watching the one `ODRA_MODULE` therefore rebuilds every
/// crate with this build script for every contract, also the crates that do not define it
/// (`odra-modules` in a project that uses it, say). So `cargo odra` also names the module in a
/// variable of the crate that defines it, and announces that with `ODRA_BUILD_PROTOCOL=2`; a
/// crate then watches its own variable only, and stays built while the contracts of other crates
/// are.
///
/// Without `ODRA_BUILD_PROTOCOL` (an older `cargo odra`, a manual build) `ODRA_MODULE` is used
/// as before.
#[derive(Debug, PartialEq, Eq)]
struct ModuleSelection {
    /// Value of the `odra_module` cfg.
    cfg: String,
    /// Variables to rebuild the crate on.
    watched: Vec<String>,
    /// Whether the crate may define the module: always under the plain `ODRA_MODULE`, which
    /// every crate sees, and only in the crate named by a per-crate variable.
    own: bool
}

impl ModuleSelection {
    fn from_env() -> Self {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let package = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
        let lib = lib_name(Path::new(&manifest_dir).join("Cargo.toml").as_path());
        Self::new(&package, lib.as_deref(), |name| std::env::var(name).ok())
    }

    /// `package` is the crate's package name and `lib` the name of its library, if the manifest
    /// gives it one. The macros gate a module on the library name (`CARGO_CRATE_NAME`), so that
    /// is the name the cfg value is built with, and the variable is read under both names.
    fn new(package: &str, lib: Option<&str>, env: impl Fn(&str) -> Option<String>) -> Self {
        let mut watched = vec![BUILD_PROTOCOL_ENV.to_string()];
        let per_crate = env(BUILD_PROTOCOL_ENV)
            .and_then(|protocol| protocol.trim().parse::<u32>().ok())
            .is_some_and(|protocol| protocol >= PER_CRATE_PROTOCOL);
        if !per_crate {
            watched.push("ODRA_MODULE".to_string());
            return Self {
                cfg: env("ODRA_MODULE").unwrap_or_default(),
                watched,
                own: true
            };
        }

        let crate_name = lib.unwrap_or(package).replace('-', "_");
        let mut variables = vec![crate_module_env(&crate_name)];
        let package_variable = crate_module_env(package);
        if !variables.contains(&package_variable) {
            variables.push(package_variable);
        }
        let module = variables
            .iter()
            .filter_map(|variable| env(variable))
            .find(|module| !module.is_empty());
        watched.extend(variables);
        match module {
            Some(module) => Self {
                cfg: format!("{}::{}", crate_name, module_file_name(&module)),
                watched,
                own: true
            },
            None => Self {
                cfg: OTHER_CRATE_MODULE.to_string(),
                watched,
                own: false
            }
        }
    }

    /// The module to build, if this crate may be the one defining it.
    fn own_module(&self) -> Option<&str> {
        (self.own && !self.cfg.is_empty()).then_some(self.cfg.as_str())
    }
}

/// The `name` of the `[lib]` section of the manifest at `manifest`, if it sets one.
///
/// A build script is told the package name but not the library name, and the two differ in a
/// crate with `[lib] name = ".."`. A line scan is enough for that one key and keeps a toml parser
/// out of every contract build.
fn lib_name(manifest: &Path) -> Option<String> {
    let content = std::fs::read_to_string(manifest).ok()?;
    lib_name_in(&content)
}

fn lib_name_in(manifest: &str) -> Option<String> {
    manifest
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != "[lib]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter_map(|line| line.strip_prefix("name"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|name| {
            let name = name.split('#').next().unwrap_or(name).trim();
            name.trim_matches(|c| c == '"' || c == '\'').to_string()
        })
        .find(|name| !name.is_empty())
}

const CONTRACT_SCHEMAS_DIR: &str = "resources/casper_contract_schemas";

/// Writes the contract schema of the module named by `ODRA_MODULE` to
/// `resources/casper_contract_schemas/<module>_schema.json`.
pub fn contract_schema<S: Serialize>(schema: S) {
    let module = std::env::var("ODRA_MODULE").expect("ODRA_MODULE environment variable is not set");

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
    let filename = Path::new(path).join(schema_file_name(module));
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

/// The schema file name of an `ODRA_MODULE` value: `erc20_schema.json` for `Erc20` as well as for
/// `odra_modules::Erc20`. Both the writer and the build-script lookup go through here, so a
/// schema written for a module is also the one found when its wasm is built.
fn schema_file_name(module: &str) -> String {
    format!("{}_schema.json", to_snake_case(module_file_name(module)))
}

/// Looks for the schema file of `module` in `manifest_dir` and its ancestors (a workspace keeps
/// the schemas at its root). Returns the path found, or the path under `manifest_dir` where
/// the schema would be written.
fn find_contract_schema(manifest_dir: &Path, module: &str) -> Result<PathBuf, PathBuf> {
    let file_name = schema_file_name(module);
    manifest_dir
        .ancestors()
        .map(|dir| dir.join(CONTRACT_SCHEMAS_DIR).join(&file_name))
        .find(|path| path.is_file())
        .ok_or_else(|| manifest_dir.join(CONTRACT_SCHEMAS_DIR).join(&file_name))
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
fn contract_schema_flags(module: Option<&str>) -> Vec<String> {
    let mut flags = vec![];
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return flags;
    }
    let Some(module) = module else {
        return flags;
    };
    let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return flags;
    };
    match find_contract_schema(Path::new(&manifest_dir), module) {
        Ok(path) => {
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
        Err(path) => {
            // Rerun once the schema gets generated.
            flags.push(format!("cargo:rerun-if-changed={}", path.display()));
            // Under the plain `ODRA_MODULE` dependencies are built with the same value, so warn
            // only in the crate that defines the contract.
            let contract = module_file_name(module);
            if !odra_toml_lists_contract(Path::new(&manifest_dir), contract) {
                return flags;
            }
            flags.push(format!(
                "cargo:warning=Contract schema for {} not found at {}. The contract will be built without an embedded schema. Run `cargo odra schema -c {}` to generate it.",
                module,
                path.display(),
                to_snake_case(contract)
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

/// The `Odra.toml` closest to `dir`: in the directory itself or in one of its ancestors (a
/// workspace keeps it at its root).
fn find_odra_toml(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .map(|dir| dir.join("Odra.toml"))
        .find(|path| path.is_file())
}

/// The `fqn` of every contract an `Odra.toml` lists.
fn odra_toml_fqns(odra_toml: &str) -> impl Iterator<Item = &str> {
    odra_toml
        .lines()
        .filter_map(|line| line.trim().strip_prefix("fqn"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|fqn| fqn.trim().trim_matches('"'))
}

/// Checks if the `Odra.toml` closest to `dir` lists a contract named `contract_name`
/// (the last segment of its `fqn`).
fn odra_toml_lists_contract(dir: &Path, contract_name: &str) -> bool {
    let Some(content) = find_odra_toml(dir).and_then(|path| std::fs::read_to_string(path).ok())
    else {
        return false;
    };
    let lists = odra_toml_fqns(&content).any(|fqn| fqn.rsplit("::").next() == Some(contract_name));
    lists
}

/// The file `odra::link_contract_crates!()` includes, written to `OUT_DIR`.
const CONTRACT_CRATES_FILE: &str = "odra_contract_crates.rs";

/// Writes [`CONTRACT_CRATES_FILE`]: an `extern crate` item for every dependency of the crate that
/// `Odra.toml` lists a contract of (`fqn = "odra_modules::erc20::Erc20"`).
///
/// Rust links a dependency only into a crate that refers to it, so a crate that builds a
/// contract of a dependency without using it gets a wasm without entry points. The
/// `bin/build_contract.rs` and `bin/build_schema.rs` include the file with
/// `odra::link_contract_crates!()`, which is that reference, kept up to date with `Odra.toml`.
///
/// The file is always written, also empty, so the `include!` never fails.
fn contract_crates_flags() -> Vec<String> {
    let mut flags = vec![];
    let (Ok(manifest_dir), Ok(out_dir)) = (
        std::env::var("CARGO_MANIFEST_DIR"),
        std::env::var("OUT_DIR")
    ) else {
        return flags;
    };
    let manifest_dir = Path::new(&manifest_dir);
    let manifest = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap_or_default();
    let odra_toml = find_odra_toml(manifest_dir);
    let contracts = odra_toml
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    if let Some(path) = &odra_toml {
        flags.push(format!("cargo:rerun-if-changed={}", path.display()));
    }

    let source = contract_crates_source(&contract_crates(&contracts, &manifest));
    let file = Path::new(&out_dir).join(CONTRACT_CRATES_FILE);
    // Left alone when it already says the same, so its mtime does not make rustc rebuild the
    // binaries that include it.
    if std::fs::read_to_string(&file).ok().as_deref() != Some(source.as_str()) {
        if let Err(err) = std::fs::write(&file, source) {
            flags.push(format!(
                "cargo:warning={} could not be written: {}",
                file.display(),
                err
            ));
        }
    }
    flags
}

/// The dependencies of the crate with the manifest `manifest` that the `Odra.toml` content
/// `odra_toml` lists a contract of, as crate names, sorted.
fn contract_crates(odra_toml: &str, manifest: &str) -> Vec<String> {
    let dependencies = dependency_names(manifest);
    let mut crates: Vec<String> = odra_toml_fqns(odra_toml)
        .filter_map(|fqn| fqn.split("::").next())
        .map(|first| first.replace('-', "_"))
        .filter(|first| dependencies.contains(first))
        .collect();
    crates.sort();
    crates.dedup();
    crates
}

fn contract_crates_source(crates: &[String]) -> String {
    let mut source = String::from(
        "// Generated by odra-build from Odra.toml: the dependency crates that define contracts.\n"
    );
    for name in crates {
        // `as _`: the binary may also name the crate itself (`use odra_modules;`).
        source.push_str(&format!("extern crate {} as _;\n", name));
    }
    source
}

/// The crate names of the `[dependencies]` of a manifest, with `-` replaced by `_`.
///
/// Only what is certainly a dependency of the crate's binaries: optional dependencies (they may
/// be disabled) and those of a `[target.*]` table (they may be for another target) are left
/// out, as an `extern crate` of a crate that is not there does not compile. A dependency that is
/// left out has to be named by hand, as before.
///
/// A line scan, like the other manifest lookups here: it keeps a toml parser out of every
/// contract build.
fn dependency_names(manifest: &str) -> Vec<String> {
    enum Section {
        Dependencies,
        /// `[dependencies.<name>]`
        Dependency,
        Other
    }
    let mut names: Vec<String> = vec![];
    let mut section = Section::Other;
    for line in manifest.lines() {
        let line = line.split('#').next().unwrap_or(line).trim();
        if let Some(header) = line.strip_prefix('[') {
            let header = header.trim_end_matches(']').trim();
            section = if header == "dependencies" {
                Section::Dependencies
            } else if let Some(name) = header.strip_prefix("dependencies.") {
                names.push(crate_name_of_key(name));
                Section::Dependency
            } else {
                Section::Other
            };
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match section {
            Section::Dependencies => {
                let optional = value.replace(' ', "").contains("optional=true")
                    || (key.ends_with(".optional") && value == "true");
                // `name = ..`, or a dotted key: `name.workspace = true`.
                let name = crate_name_of_key(key.split('.').next().unwrap_or(key));
                if optional {
                    names.retain(|known| *known != name);
                } else if !names.contains(&name) {
                    names.push(name);
                }
            }
            Section::Dependency if key == "optional" && value == "true" => {
                names.pop();
            }
            _ => {}
        }
    }
    names
}

fn crate_name_of_key(key: &str) -> String {
    key.trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .replace('-', "_")
}

fn flags(selection: &ModuleSelection, backend: Option<String>) -> Vec<String> {
    let mut flags: Vec<String> = selection
        .watched
        .iter()
        .map(|variable| format!("cargo:rerun-if-env-changed={}", variable))
        .collect();
    flags.push(format!("cargo:rustc-cfg=odra_module=\"{}\"", selection.cfg));

    flags.push("cargo:rerun-if-env-changed=ODRA_BACKEND".to_string());
    flags.push(format!(
        "cargo:rustc-cfg=odra_backend=\"{}\"",
        backend.unwrap_or_default()
    ));

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

    fn selection(package: &str, lib: Option<&str>, env: &[(&str, &str)]) -> super::ModuleSelection {
        super::ModuleSelection::new(package, lib, |name| {
            env.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        })
    }

    #[test]
    fn test_flags() {
        let flags = super::flags(&selection("my-token", None, &[]), None);
        assert_eq!(
            flags,
            vec![
                "cargo:rerun-if-env-changed=ODRA_BUILD_PROTOCOL",
                "cargo:rerun-if-env-changed=ODRA_MODULE",
                "cargo:rustc-cfg=odra_module=\"\"",
                "cargo:rerun-if-env-changed=ODRA_BACKEND",
                "cargo:rustc-cfg=odra_backend=\"\""
            ]
        );

        let flags = super::flags(
            &selection("my-token", None, &[("ODRA_MODULE", "test")]),
            Some("backend_test".to_string())
        );
        assert_eq!(flags[2], "cargo:rustc-cfg=odra_module=\"test\"");
        assert_eq!(flags[4], "cargo:rustc-cfg=odra_backend=\"backend_test\"");
    }

    /// Without `ODRA_BUILD_PROTOCOL` (or with a value older than the per-crate one) the crate
    /// is built for `ODRA_MODULE`, whatever crate it names.
    #[test]
    fn test_module_selection_plain() {
        for protocol in [None, Some("1"), Some(""), Some("two")] {
            let mut env = vec![
                ("ODRA_MODULE", "odra_modules::Erc20"),
                ("ODRA_MODULE__my_token", "Token"),
            ];
            if let Some(protocol) = protocol {
                env.push(("ODRA_BUILD_PROTOCOL", protocol));
            }
            let selection = selection("my-token", None, &env);
            assert_eq!(selection.cfg, "odra_modules::Erc20");
            assert_eq!(selection.watched, ["ODRA_BUILD_PROTOCOL", "ODRA_MODULE"]);
            assert_eq!(selection.own_module(), Some("odra_modules::Erc20"));
        }
        assert_eq!(selection("my-token", None, &[]).own_module(), None);
    }

    /// With `ODRA_BUILD_PROTOCOL=2` a crate watches its own variable instead of `ODRA_MODULE`,
    /// and one that does not define the contract gets the same placeholder for every contract.
    #[test]
    fn test_module_selection_per_crate() {
        let env = [
            ("ODRA_BUILD_PROTOCOL", "2"),
            ("ODRA_MODULE", "my_token::Token"),
            ("ODRA_MODULE__my_token", "Token")
        ];
        let defining = selection("my-token", None, &env);
        assert_eq!(defining.cfg, "my_token::Token");
        assert_eq!(
            defining.watched,
            ["ODRA_BUILD_PROTOCOL", "ODRA_MODULE__my_token"]
        );
        assert_eq!(defining.own_module(), Some("my_token::Token"));

        let other = selection("odra-modules", None, &env);
        assert_eq!(other.cfg, "_");
        assert_eq!(
            other.watched,
            ["ODRA_BUILD_PROTOCOL", "ODRA_MODULE__odra_modules"]
        );
        assert_eq!(other.own_module(), None);

        // A crate-qualified value names the same module.
        let qualified = [
            ("ODRA_BUILD_PROTOCOL", "3"),
            ("ODRA_MODULE__my_token", "my_token::Token")
        ];
        assert_eq!(
            selection("my-token", None, &qualified).cfg,
            "my_token::Token"
        );
        // An empty variable selects nothing.
        let empty = [("ODRA_BUILD_PROTOCOL", "2"), ("ODRA_MODULE__my_token", "")];
        assert_eq!(selection("my-token", None, &empty).cfg, "_");
    }

    /// The macros gate a module on the library name, so the cfg value is built with it; the
    /// variable may be named after the library or the package.
    #[test]
    fn test_module_selection_with_lib_name() {
        for variable in ["ODRA_MODULE__token_lib", "ODRA_MODULE__my_token"] {
            let env = [("ODRA_BUILD_PROTOCOL", "2"), (variable, "Token")];
            let selection = selection("my-token", Some("token_lib"), &env);
            assert_eq!(selection.cfg, "token_lib::Token");
            assert_eq!(
                selection.watched,
                [
                    "ODRA_BUILD_PROTOCOL",
                    "ODRA_MODULE__token_lib",
                    "ODRA_MODULE__my_token"
                ]
            );
        }
    }

    #[test]
    fn test_lib_name_in() {
        let name = super::lib_name_in;
        assert_eq!(name("[package]\nname = \"my-token\"\n"), None);
        assert_eq!(
            name("[package]\nname = \"my-token\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n"),
            None
        );
        assert_eq!(
            name("[package]\nname = \"my-token\"\n[lib]\npath = \"src/lib.rs\"\nname = \"token_lib\" # the lib\n"),
            Some("token_lib".to_string())
        );
        assert_eq!(
            name("[lib]\nname=\'token_lib\'\n[[bin]]\nname = \"my_token_build_contract\"\n"),
            Some("token_lib".to_string())
        );
        // Only the `[lib]` section counts.
        assert_eq!(
            name("[lib]\ntest = false\n[[bin]]\nname = \"my_token_build_contract\"\n"),
            None
        );
    }

    #[test]
    fn test_dependency_names() {
        let manifest = r#"
[package]
name = "my-token"   # not a dependency

[dependencies]
odra = { workspace = true }
odra-modules = "2.11.0" # with a comment
"quoted-name" = { version = "1", features = [
    "std",
] }
dotted.workspace = true
optional-dep = { version = "1", optional = true }
dotted-optional.version = "1"
dotted-optional.optional = true

[dependencies.table-dep]
version = "1"

[dependencies.optional-table-dep]
version = "1"
optional = true

[dev-dependencies]
odra-test = "2.11.0"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
odra-build = { workspace = true }

[[bin]]
name = "my_token_build_contract"
"#;
        assert_eq!(
            super::dependency_names(manifest),
            ["odra", "odra_modules", "quoted_name", "dotted", "table_dep"]
        );
    }

    #[test]
    fn test_contract_crates() {
        let manifest = "[dependencies]\nodra = \"2\"\nodra-modules = \"2\"\nother = \"1\"\n";
        let odra_toml = r#"
[[contracts]]
fqn = "my_token::token::Token"

[[contracts]]
fqn = "odra_modules::erc20::Erc20"

[[contracts]]
fqn = "odra_modules::cep18_token::Cep18"

[[contracts]]
fqn = "not_a_dependency::Contract"

[[contracts]]
fqn = "Bare"
"#;
        let crates = super::contract_crates(odra_toml, manifest);
        assert_eq!(crates, ["odra_modules"]);
        assert_eq!(
            super::contract_crates_source(&crates)
                .lines()
                .skip(1)
                .collect::<Vec<_>>(),
            ["extern crate odra_modules as _;"]
        );
        assert!(super::contract_crates("", manifest).is_empty());
        assert_eq!(super::contract_crates_source(&[]).lines().count(), 1);
    }

    #[test]
    fn test_schema_file_name() {
        let name = super::schema_file_name;
        assert_eq!(name("Erc20"), "erc20_schema.json");
        assert_eq!(name("odra_modules::Erc20"), "erc20_schema.json");
        assert_eq!(name("odra_modules::Cep18"), "cep18_schema.json");
        assert_eq!(
            name("odra_examples::PartyContract"),
            "party_contract_schema.json"
        );
    }

    /// The build script looks the schema up under the crate-qualified `ODRA_MODULE` that
    /// `cargo odra` passes; snake-casing the whole value used to look for
    /// `odra_modules::_erc20_schema.json` and silently build without the schema.
    #[test]
    fn test_find_contract_schema() {
        let root = std::env::temp_dir().join(format!("odra-build-find-{}", std::process::id()));
        let member = root.join("member");
        let schemas = root.join(super::CONTRACT_SCHEMAS_DIR);
        std::fs::create_dir_all(&member).unwrap();
        std::fs::create_dir_all(&schemas).unwrap();
        let erc20 = schemas.join("erc20_schema.json");
        std::fs::write(&erc20, "{}").unwrap();

        // Found under the bare and the qualified name, from the root and from a member.
        assert_eq!(
            super::find_contract_schema(&root, "Erc20"),
            Ok(erc20.clone())
        );
        assert_eq!(
            super::find_contract_schema(&root, "odra_modules::Erc20"),
            Ok(erc20.clone())
        );
        assert_eq!(
            super::find_contract_schema(&member, "odra_modules::Erc20"),
            Ok(erc20)
        );
        // Missing: the path the schema would be written to, in the crate being built.
        assert_eq!(
            super::find_contract_schema(&member, "odra_modules::Cep18"),
            Err(member
                .join(super::CONTRACT_SCHEMAS_DIR)
                .join("cep18_schema.json"))
        );

        std::fs::remove_dir_all(&root).unwrap();
    }
}
