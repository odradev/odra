use std::fmt;

use casper_hashing::Digest;
use casper_types::bytesrepr::{self, FromBytes, ToBytes};
use datasize::DataSize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A cryptographic hash identifying a [`Block`](struct.Block.html).
#[derive(
    Copy,
    Clone,
    DataSize,
    Default,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
    Debug,
    JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BlockHash(Digest);

impl fmt::Display for BlockHash {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "block hash {}", self.0)
    }
}

impl From<Digest> for BlockHash {
    fn from(digest: Digest) -> Self {
        Self(digest)
    }
}

impl AsRef<[u8]> for BlockHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl ToBytes for BlockHash {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for BlockHash {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (hash, remainder) = Digest::from_bytes(bytes)?;
        let block_hash = BlockHash(hash);
        Ok((block_hash, remainder))
    }
}

/// Describes a block's hash and height.
#[derive(
    Clone, Copy, DataSize, Default, Eq, JsonSchema, Serialize, Deserialize, Debug, PartialEq,
)]
pub struct BlockHashAndHeight {
    /// The hash of the block.
    #[schemars(description = "The hash of this deploy's block.")]
    pub block_hash: BlockHash,
    /// The height of the block.
    #[schemars(description = "The height of this deploy's block.")]
    pub block_height: u64
}

impl fmt::Display for BlockHashAndHeight {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}, height {} ",
            self.block_hash, self.block_height
        )
    }
}
