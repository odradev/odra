use crate::log;
use casper_types::bytesrepr::FromBytes;
use casper_types::StoredValue::CLValue;
use casper_types::{CLTyped, StoredValue};
use odra_core::prelude::{ExecutionError, OdraError, OdraResult};
use std::path::{self, PathBuf};
use std::time::Duration;

use crate::error::LivenetError;

/// Search for the wasm file in the current directory and in the parent directory.
pub fn find_wasm_file_path(wasm_file_name: &str) -> OdraResult<PathBuf> {
    let contract_path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");

    let project_root = project_root::get_project_root().map_err(|e| {
        OdraError::ExecutionError(ExecutionError::ContractDeploymentError(e.to_string()))
    })?;
    let mut current_dir = path::absolute(".").map_err(|e| {
        OdraError::ExecutionError(ExecutionError::ContractDeploymentError(e.to_string()))
    })?;

    let mut checked_paths = vec![];
    while current_dir != project_root {
        let path = current_dir.join(&contract_path);
        if path.exists() {
            crate::log::info(format!("Found wasm under {:?}.", path));
            return Ok(path);
        } else {
            checked_paths.push(path);
            current_dir = current_dir
                .parent()
                .ok_or(OdraError::ExecutionError(
                    ExecutionError::ContractDeploymentError(
                        "Failed to get parent directory".to_string()
                    )
                ))?
                .to_path_buf();
        }
    }
    let path = current_dir.join(&contract_path);
    checked_paths.push(path.clone());
    if path.exists() {
        crate::log::info(format!("Found wasm under {:?}.", path));
        return Ok(path);
    }

    crate::log::error(format!("Could not find wasm under {:?}.", checked_paths));
    Err(OdraError::ExecutionError(
        ExecutionError::ContractDeploymentError("Failed to find wasm file".to_string())
    ))
}

/// Gets an env variable
pub fn get_env_variable(name: &str) -> Result<String, LivenetError> {
    std::env::var(name).map_err(|_err| LivenetError::EnvVariableNotSet(name.to_string()))
}

/// Gets an optional env variable
pub fn get_optional_env_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

pub fn extract_stored_value<T: CLTyped + FromBytes>(value: StoredValue) -> T {
    match value {
        CLValue(value) => value
            .clone()
            .into_t()
            .unwrap_or_else(|_| panic!("Couldn't get bytes from CLValue: {:?}", value)),
        _ => panic!("Value stored in result key is not a CLValue")
    }
}

/// Number of attempts for an RPC call rejected with HTTP 429 (Too Many Requests).
const RATE_LIMIT_ATTEMPTS: u32 = 5;
/// Delay before the first retry; doubled after each further 429.
const RATE_LIMIT_BASE_DELAY: Duration = Duration::from_millis(200);

/// Errors that can tell whether the node (or its sidecar) rate-limited the request.
pub trait RateLimited {
    fn is_rate_limited(&self) -> bool;
}

impl RateLimited for casper_client::Error {
    fn is_rate_limited(&self) -> bool {
        match self {
            // The sidecar itself refuses the request.
            casper_client::Error::ResponseIsHttpError { error, .. }
            | casper_client::Error::FailedToGetResponse { error, .. } => {
                error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
            }
            // The sidecar accepted it but the node behind it did not:
            // {"code":-32018,"message":"Node request failure","data":"...: request was throttled by the node"}
            casper_client::Error::ResponseIsRpcError { error, .. } => error
                .data
                .as_ref()
                .is_some_and(|data| data.to_string().contains("throttled")),
            _ => false
        }
    }
}

impl RateLimited for casper_client::cli::CliError {
    fn is_rate_limited(&self) -> bool {
        matches!(self, casper_client::cli::CliError::Core(e) if e.is_rate_limited())
    }
}

/// Runs an RPC call, retrying with exponential backoff while the node answers HTTP 429.
///
/// Nodes and sidecars (NCTL, cspr.cloud) rate-limit JSON-RPC; a burst of reads from a test or a
/// script otherwise fails on the second or third request. Any other error is returned as is.
pub async fn retry_on_rate_limit<T, E, F, Fut>(what: &str, call: F) -> Result<T, E>
where
    E: RateLimited + core::fmt::Display,
    F: Fn() -> Fut,
    Fut: core::future::Future<Output = Result<T, E>>
{
    let mut delay = RATE_LIMIT_BASE_DELAY;
    for attempt in 1..RATE_LIMIT_ATTEMPTS {
        match call().await {
            Err(e) if e.is_rate_limited() => {
                log::debug(format!(
                    "{what}: rate limited by the node (attempt {attempt}/{RATE_LIMIT_ATTEMPTS}), retrying in {delay:?}"
                ));
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            result => return result
        }
    }
    call().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug)]
    enum TestError {
        RateLimited,
        Other
    }

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl RateLimited for TestError {
        fn is_rate_limited(&self) -> bool {
            matches!(self, TestError::RateLimited)
        }
    }

    fn run<T>(fut: impl core::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
            .block_on(fut)
    }

    #[test]
    fn retries_while_rate_limited() {
        let calls = Cell::new(0);
        let result = run(retry_on_rate_limit("test", || {
            calls.set(calls.get() + 1);
            let n = calls.get();
            async move {
                if n < 3 {
                    Err(TestError::RateLimited)
                } else {
                    Ok(n)
                }
            }
        }));
        assert_eq!(result.unwrap(), 3);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn gives_up_after_the_last_attempt() {
        let calls = Cell::new(0);
        let result: Result<(), TestError> = run(retry_on_rate_limit("test", || {
            calls.set(calls.get() + 1);
            async { Err(TestError::RateLimited) }
        }));
        assert!(matches!(result, Err(TestError::RateLimited)));
        assert_eq!(calls.get(), RATE_LIMIT_ATTEMPTS);
    }

    #[test]
    fn other_errors_are_not_retried() {
        let calls = Cell::new(0);
        let result: Result<(), TestError> = run(retry_on_rate_limit("test", || {
            calls.set(calls.get() + 1);
            async { Err(TestError::Other) }
        }));
        assert!(matches!(result, Err(TestError::Other)));
        assert_eq!(calls.get(), 1);
    }
}
