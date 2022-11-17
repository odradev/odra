use borsh::{BorshSerialize, BorshDeserialize};

/// A wrapper for bytes that has efficient (de)serialization.
#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct Bytes(Vec<u8>);
