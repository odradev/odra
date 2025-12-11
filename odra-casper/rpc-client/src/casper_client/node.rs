//! Node state and configuration methods.

use crate::casper_client::Result;
use crate::error::LivenetError::BlockTimeError;
use casper_client::cli::{get_node_status, get_state_root_hash};
use casper_client::get_chainspec;
use casper_types::{Digest, TimeDiff};
use std::str::FromStr;
use toml::Value;

/// Node-related methods implementation for CasperClient.
impl super::CasperClient {
    /// Returns the current block_time
    pub async fn get_block_time(&self) -> Result<u64> {
        let block_time = get_node_status(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity()
        )
        .await
        .map_err(|_| BlockTimeError)?
        .result
        .last_added_block_info
        .ok_or(BlockTimeError)?
        .timestamp
        .millis();
        Ok(block_time)
    }

    /// Query the node for the current state root hash.
    pub async fn get_state_root_hash(&self) -> Result<String> {
        let digest = self.get_state_root_hash_digest().await?;
        Ok(base16::encode_lower(&digest))
    }

    pub async fn get_state_root_hash_digest(&self) -> Result<Digest> {
        let response = get_state_root_hash(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity(),
            ""
        )
        .await
        .map_err(|e| {
            crate::error::LivenetError::ClientError(format!(
                "Couldn't get state root hash from node: {:?}, error: {}",
                self.configuration.node_address(),
                e
            ))
        })?;
        response.result.state_root_hash.ok_or_else(|| {
            crate::error::LivenetError::ClientError(format!(
                "State root hash not available from node: {:?}",
                self.configuration.node_address()
            ))
        })
    }

    /// Gets the chainspec from the node.
    pub(crate) async fn chainspec(&self) -> Value {
        let chainspec = get_chainspec(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed()
        )
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Couldn't get chainspec from node: {:?}, reason: {:?}",
                self.configuration.node_address(),
                e
            )
        })
        .result;
        let toml_bytes: &[u8] = chainspec.chainspec_bytes.chainspec_bytes();
        let toml = String::from_utf8(toml_bytes.to_vec()).unwrap();
        toml::from_str(&toml).unwrap_or_else(|e| panic!("Couldn't parse chainspec bytes: {:?}", e))
    }

    /// Extracts era duration from chainspec.
    pub(crate) fn era_duration(chainspec: &Value) -> Result<u64> {
        let era_duration = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get core from chainspec"))
            .get("era_duration")
            .unwrap_or_else(|| {
                panic!("Couldn't get era_duration from chainspec");
            });
        let era_duration_str = era_duration.as_str().ok_or_else(|| {
            crate::error::LivenetError::ClientError(format!(
                "era_duration is not a string in chainspec: {:?}",
                era_duration
            ))
        })?;
        let time_diff = TimeDiff::from_str(era_duration_str).map_err(|e| {
            crate::error::LivenetError::ClientError(format!(
                "Couldn't parse era_duration from chainspec: {:?}, error: {}",
                era_duration, e
            ))
        })?;
        Ok(time_diff.millis())
    }
}
