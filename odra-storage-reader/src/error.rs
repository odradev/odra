use thiserror::Error;

use odra_schema::STORAGE_LAYOUT_VERSION;

/// Errors of the storage reader. In the wasm build they are thrown as `JsError`s.
#[derive(Debug, Error)]
pub enum Error {
    /// The layout file has no `version` field.
    #[error("The layout file has no `version`, export it again with `odra-cli --json storage <Contract>`")]
    MissingVersion,
    /// The layout file has a version this reader does not support.
    #[error("Unsupported layout file version {0}, expected {STORAGE_LAYOUT_VERSION}")]
    UnsupportedVersion(u64),
    /// The layout file does not match the expected format.
    #[error("Invalid layout file: {0}")]
    InvalidLayout(String),
    /// The path cannot be resolved against the layout, or the keys do not match it.
    #[error("Cannot resolve `{path}`: {reason}")]
    Resolve {
        /// The requested path.
        path: String,
        /// Why it cannot be resolved.
        reason: String
    },
    /// The given location is not one returned by `locate`.
    #[error("Invalid location: {0}")]
    InvalidLocation(String),
    /// The given value is not a node `CLValue` (`stored_value.CLValue`).
    #[error("Invalid CLValue: {0}")]
    InvalidClValue(String),
    /// The stored bytes cannot be decoded as the expected type.
    #[error("Cannot decode the stored value: {reason} (raw: 0x{raw})")]
    Decode {
        /// Why it cannot be decoded.
        reason: String,
        /// The hex-encoded stored bytes.
        raw: String
    },
    /// The given value is not a contract package (`stored_value` of the package hash).
    #[error("Invalid contract package: {0}")]
    InvalidPackage(String)
}
