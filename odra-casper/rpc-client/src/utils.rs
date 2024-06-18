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

pub fn get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        crate::log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

pub fn get_optional_env_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
