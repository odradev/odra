/// A wrapper for bytes that has efficient (de)serialization.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Bytes(Vec<u8>);
