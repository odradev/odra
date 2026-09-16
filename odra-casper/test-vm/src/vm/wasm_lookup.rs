//! Locating contract wasm files for the Casper test VM.
//!
//! Cargo runs a crate's tests inside that crate's directory. In a workspace, `cargo odra build`
//! puts the wasm files in the `wasm` directory at the workspace root, so a test that deploys a
//! contract has to look above its own crate to find them.

use std::path::{Path, PathBuf};

const WASM_DIR: &str = "wasm";

/// Reads `wasm/<file_name>`, looking in the working directory first and then in each of its
/// ancestors, so a single `wasm` directory at the workspace root serves every member.
///
/// Panics with the list of directories searched if the file is not found, since a missing
/// wasm file is a test set-up problem rather than a contract failure.
pub fn read_wasm_file(file_name: &str) -> Vec<u8> {
    let cwd = std::env::current_dir().expect("should get current working dir");
    let Some(path) = find_wasm_file(&cwd, file_name) else {
        let searched = cwd
            .ancestors()
            .map(|dir| dir.join(WASM_DIR).display().to_string())
            .collect::<Vec<_>>()
            .join("\n  ");
        panic!(
            "Couldn't find `{file_name}`. Run `cargo odra build` first. Looked in:\n  {searched}"
        );
    };
    std::fs::read(&path).unwrap_or_else(|err| panic!("Couldn't read `{}`: {err}", path.display()))
}

/// Returns the first `<dir>/wasm/<file_name>` that exists, checking `start` and then each of
/// its ancestors in turn. The nearest directory wins, so a member can shadow the root's copy.
pub fn find_wasm_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|dir| dir.join(WASM_DIR).join(file_name))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path =
                std::env::temp_dir().join(format!("odra-wasm-lookup-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn touch(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"\0asm").unwrap();
            path
        }

        fn dir(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            fs::create_dir_all(&path).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn finds_the_file_in_the_working_directory() {
        let tmp = TempDir::new();
        let expected = tmp.touch("wasm/Flipper.wasm");
        assert_eq!(find_wasm_file(&tmp.0, "Flipper.wasm"), Some(expected));
    }

    #[test]
    fn walks_up_to_the_workspace_root_from_a_member() {
        let tmp = TempDir::new();
        let expected = tmp.touch("wasm/Flipper.wasm");
        let member = tmp.dir("cli");
        assert_eq!(find_wasm_file(&member, "Flipper.wasm"), Some(expected));
    }

    #[test]
    fn the_nearest_directory_wins() {
        let tmp = TempDir::new();
        tmp.touch("wasm/Flipper.wasm");
        let member_copy = tmp.touch("flipper/wasm/Flipper.wasm");
        let member = tmp.0.join("flipper");
        assert_eq!(find_wasm_file(&member, "Flipper.wasm"), Some(member_copy));
    }

    #[test]
    fn reports_a_missing_file() {
        let tmp = TempDir::new();
        tmp.touch("wasm/Other.wasm");
        assert_eq!(find_wasm_file(&tmp.0, "Flipper.wasm"), None);
    }
}
