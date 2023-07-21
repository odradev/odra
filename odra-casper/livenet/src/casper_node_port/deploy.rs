#![allow(clippy::field_reassign_with_default)]

use std::{cell::OnceCell, cmp, collections::BTreeSet, fmt, hash};

use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    PublicKey, SecretKey
};
use datasize::DataSize;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::casper_node_port::deploy_item::DeployItem;
use crate::casper_node_port::executable_deploy_item::ExecutableDeployItem;
use crate::casper_node_port::hashing::Digest;

use crate::casper_node_port::utils::DisplayIter;
use crate::casper_types_port::timestamp::{TimeDiff, Timestamp};

use super::{
    approval::Approval, deploy_hash::DeployHash, deploy_header::DeployHeader,
    error::DeployConfigurationFailure, utils::ds
};

/// A deploy; an item containing a smart contract along with the requester's signature(s).
#[derive(Clone, DataSize, Eq, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Deploy {
    hash: DeployHash,
    header: DeployHeader,
    payment: ExecutableDeployItem,
    session: ExecutableDeployItem,
    approvals: BTreeSet<Approval>,
    #[serde(skip)]
    #[data_size(with = ds::once_cell)]
    is_valid: OnceCell<Result<(), DeployConfigurationFailure>>
}

impl Deploy {
    /// Constructs a new signed `Deploy`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: Timestamp,
        ttl: TimeDiff,
        gas_price: u64,
        dependencies: Vec<DeployHash>,
        chain_name: String,
        payment: ExecutableDeployItem,
        session: ExecutableDeployItem,
        secret_key: &SecretKey,
        account: Option<PublicKey>
    ) -> Deploy {
        let serialized_body = serialize_body(&payment, &session);
        let body_hash = Digest::hash(serialized_body);

        let account = account.unwrap_or_else(|| PublicKey::from(secret_key));

        // Remove duplicates.
        let dependencies = dependencies.into_iter().unique().collect();
        let header = DeployHeader::new(
            account,
            timestamp,
            ttl,
            gas_price,
            body_hash,
            dependencies,
            chain_name
        );
        let serialized_header = serialize_header(&header);
        let hash = DeployHash::new(Digest::hash(serialized_header));

        let mut deploy = Deploy {
            hash,
            header,
            payment,
            session,
            approvals: BTreeSet::new(),
            is_valid: OnceCell::new()
        };

        deploy.sign(secret_key);
        deploy
    }

    /// Adds a signature of this deploy's hash to its approvals.
    pub fn sign(&mut self, secret_key: &SecretKey) {
        let approval = Approval::create(&self.hash, secret_key);
        self.approvals.insert(approval);
    }

    /// Returns the `DeployHash` identifying this `Deploy`.
    pub fn hash(&self) -> &DeployHash {
        &self.hash
    }

    /// Returns a reference to the `DeployHeader` of this `Deploy`.
    pub fn header(&self) -> &DeployHeader {
        &self.header
    }

    /// Returns the `DeployHeader` of this `Deploy`.
    pub fn take_header(self) -> DeployHeader {
        self.header
    }

    /// Returns the `ExecutableDeployItem` for payment code.
    pub fn payment(&self) -> &ExecutableDeployItem {
        &self.payment
    }

    /// Returns the `ExecutableDeployItem` for session code.
    pub fn session(&self) -> &ExecutableDeployItem {
        &self.session
    }

    /// Returns the `Approval`s for this deploy.
    pub fn approvals(&self) -> &BTreeSet<Approval> {
        &self.approvals
    }
}

impl hash::Hash for Deploy {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // Destructure to make sure we don't accidentally omit fields.
        let Deploy {
            hash,
            header,
            payment,
            session,
            approvals,
            is_valid: _
        } = self;
        hash.hash(state);
        header.hash(state);
        payment.hash(state);
        session.hash(state);
        approvals.hash(state);
    }
}

impl PartialEq for Deploy {
    fn eq(&self, other: &Deploy) -> bool {
        // Destructure to make sure we don't accidentally omit fields.
        let Deploy {
            hash,
            header,
            payment,
            session,
            approvals,
            is_valid: _
        } = self;
        *hash == other.hash
            && *header == other.header
            && *payment == other.payment
            && *session == other.session
            && *approvals == other.approvals
    }
}

impl Ord for Deploy {
    fn cmp(&self, other: &Deploy) -> cmp::Ordering {
        // Destructure to make sure we don't accidentally omit fields.
        let Deploy {
            hash,
            header,
            payment,
            session,
            approvals,
            is_valid: _
        } = self;
        hash.cmp(&other.hash)
            .then_with(|| header.cmp(&other.header))
            .then_with(|| payment.cmp(&other.payment))
            .then_with(|| session.cmp(&other.session))
            .then_with(|| approvals.cmp(&other.approvals))
    }
}

impl PartialOrd for Deploy {
    fn partial_cmp(&self, other: &Deploy) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl ToBytes for Deploy {
    fn write_bytes(&self, writer: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        self.header.write_bytes(writer)?;
        self.hash.write_bytes(writer)?;
        self.payment.write_bytes(writer)?;
        self.session.write_bytes(writer)?;
        self.approvals.write_bytes(writer)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut buffer = bytesrepr::allocate_buffer(self)?;
        self.write_bytes(&mut buffer)?;
        Ok(buffer)
    }

    fn serialized_length(&self) -> usize {
        self.header.serialized_length()
            + self.hash.serialized_length()
            + self.payment.serialized_length()
            + self.session.serialized_length()
            + self.approvals.serialized_length()
    }
}

impl FromBytes for Deploy {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (header, remainder) = DeployHeader::from_bytes(bytes)?;
        let (hash, remainder) = DeployHash::from_bytes(remainder)?;
        let (payment, remainder) = ExecutableDeployItem::from_bytes(remainder)?;
        let (session, remainder) = ExecutableDeployItem::from_bytes(remainder)?;
        let (approvals, remainder) = BTreeSet::<Approval>::from_bytes(remainder)?;
        let maybe_valid_deploy = Deploy {
            header,
            hash,
            payment,
            session,
            approvals,
            is_valid: OnceCell::new()
        };
        Ok((maybe_valid_deploy, remainder))
    }
}


impl From<Deploy> for DeployItem {
    fn from(deploy: Deploy) -> Self {
        let address = deploy.header().account().to_account_hash();
        let authorization_keys = deploy
            .approvals()
            .iter()
            .map(|approval| approval.signer().to_account_hash())
            .collect();

        DeployItem::new(
            address,
            deploy.session().clone(),
            deploy.payment().clone(),
            deploy.header().gas_price(),
            authorization_keys,
            casper_types::DeployHash::new(deploy.hash().inner().value())
        )
    }
}

fn serialize_header(header: &DeployHeader) -> Vec<u8> {
    header
        .to_bytes()
        .unwrap_or_else(|error| panic!("should serialize deploy header: {}", error))
}

fn serialize_body(payment: &ExecutableDeployItem, session: &ExecutableDeployItem) -> Vec<u8> {
    let mut buffer = payment
        .to_bytes()
        .unwrap_or_else(|error| panic!("should serialize payment code: {}", error));
    buffer.extend(
        session
            .to_bytes()
            .unwrap_or_else(|error| panic!("should serialize session code: {}", error))
    );
    buffer
}
