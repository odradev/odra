use thiserror::Error;

pub type Result<T> = std::result::Result<T, CodegenError>;

#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid schema path")]
    InvalidSchemaPath,
    #[error("Failed to deserialize schema JSON: {0}")]
    SchemaDeserializationError(#[from] serde_json::Error)
}
