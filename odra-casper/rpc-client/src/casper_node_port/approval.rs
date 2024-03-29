use std::fmt;

use datasize::DataSize;
use odra_core::casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    crypto, PublicKey, SecretKey, Signature
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::deploy_hash::DeployHash;

/// A struct containing a signature of a deploy hash and the public key of the signer.
#[derive(
    Clone, DataSize, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    signer: PublicKey,
    signature: Signature
}

impl Approval {
    /// Creates an approval for the given deploy hash using the given secret key.
    pub fn create(hash: &DeployHash, secret_key: &SecretKey) -> Self {
        let signer = PublicKey::from(secret_key);
        let signature = crypto::sign(hash, secret_key, &signer);
        Self { signer, signature }
    }

    /// Returns the public key of the approval's signer.
    pub fn signer(&self) -> &PublicKey {
        &self.signer
    }

    /// Returns the approval signature.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl fmt::Display for Approval {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "approval({})", self.signer)
    }
}

impl ToBytes for Approval {
    fn write_bytes(&self, writer: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        self.signer.write_bytes(writer)?;
        self.signature.write_bytes(writer)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        self.write_bytes(&mut buffer)?;
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        self.signer.serialized_length() + self.signature.serialized_length()
    }
}

impl FromBytes for Approval {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (signer, remainder) = PublicKey::from_bytes(bytes)?;
        let (signature, remainder) = Signature::from_bytes(remainder)?;
        let approval = Approval { signer, signature };
        Ok((approval, remainder))
    }
}
