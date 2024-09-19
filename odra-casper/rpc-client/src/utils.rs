use serde_json::Value;
use std::path::PathBuf;

/// Search for the wasm file in the current directory and in the parent directory.
pub fn find_wasm_file_path(wasm_file_name: &str) -> PathBuf {
    let mut path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");
    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() && path.is_file() {
            crate::log::info(format!("Found wasm under {:?}.", path));
            return path;
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    crate::log::error(format!("Could not find wasm under {:?}.", checked_paths));
    panic!("Wasm not found");
}

/// Gets an env variable
pub fn get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        crate::log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

/// Gets an optional env variable
pub fn get_optional_env_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Converts RuntimeArgs into Vec<String> compatible with rustSDK
pub fn runtime_args_to_simple_args(runtime_args: &casper_types::RuntimeArgs) -> Vec<String> {
    runtime_args
        .named_args()
        .map(|named_arg| {
            let value = serde_json::to_string(&named_arg.cl_value()).unwrap();
            let json: Value = serde_json::from_str(&value).unwrap();
            let value = json.get("parsed").unwrap().to_string();
            format!(
                "{}:{}='{}'",
                named_arg.name(),
                named_arg.cl_value().cl_type(),
                value,
            )
        })
        .collect()
}
