use std::{collections::BTreeSet, fmt, str::FromStr};

use casper_types::{
    Approval, AsymmetricType, DeployHash, Digest, PublicKey, SecretKey, Signature, Transaction,
    TransactionHash, TransactionV1, TransactionV1Hash
};

use crate::{Error, MAX_APPROVALS};

const PREFIX: &str = "odra-sig";

/// Signs the transaction's hash with `secret_key`. The transaction itself does not change; add
/// the approval with [`merge`], or send it as a [`SignatureString`].
pub fn sign(transaction: &Transaction, secret_key: &SecretKey) -> Approval {
    Approval::create(&transaction.hash(), secret_key)
}

/// Checks that `approval` is a valid signature of `hash`.
pub fn verify_approval(hash: &TransactionHash, approval: &Approval) -> Result<(), Error> {
    casper_types::crypto::verify(hash, approval.signature(), approval.signer())
        .map_err(|_| Error::BadSignature(approval.signer().to_hex()))
}

/// Checks the hash and every approval of a transaction. Unlike `Transaction::verify`, a
/// transaction without approvals is fine: it is still being signed.
pub(crate) fn verify_transaction(transaction: &Transaction) -> Result<(), Error> {
    let approvals = transaction.approvals();
    if approvals.len() > MAX_APPROVALS {
        return Err(Error::TooManyApprovals(approvals.len()));
    }
    if approvals.is_empty() {
        return match transaction {
            Transaction::V1(v1) => v1
                .has_valid_hash()
                .map_err(|error| Error::Invalid(error.to_string())),
            Transaction::Deploy(deploy) => deploy
                .has_valid_hash()
                .map_err(|error| Error::Invalid(error.to_string()))
        };
    }
    transaction
        .verify()
        .map_err(|error| Error::Invalid(error.to_string()))
}

/// Adds `approvals` to the transaction. Every approval must be a valid signature of this
/// transaction's hash; approvals it already has are kept once. The hash does not change.
pub fn merge(
    transaction: Transaction,
    approvals: impl IntoIterator<Item = Approval>
) -> Result<Transaction, Error> {
    let hash = transaction.hash();
    let mut all: BTreeSet<Approval> = transaction.approvals();
    for approval in approvals {
        verify_approval(&hash, &approval)?;
        all.insert(approval);
    }
    if all.len() > MAX_APPROVALS {
        return Err(Error::TooManyApprovals(all.len()));
    }
    let merged = match transaction {
        Transaction::V1(v1) => {
            Transaction::V1(TransactionV1::new(*v1.hash(), v1.payload().clone(), all))
        }
        // A Deploy has no public constructor taking approvals; `with_approvals` is what the node
        // itself uses to put finalized approvals back on a deploy.
        Transaction::Deploy(deploy) => Transaction::Deploy(deploy.with_approvals(all))
    };
    verify_transaction(&merged)?;
    Ok(merged)
}

/// One approval of one transaction as a line of text:
/// `odra-sig:<v1|deploy>:<transaction hash>:<public key>:<signature>`, all hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureString {
    /// The transaction signed.
    pub hash: TransactionHash,
    /// The signature.
    pub approval: Approval
}

impl SignatureString {
    /// The approval of `hash`, checked.
    pub fn new(hash: TransactionHash, approval: Approval) -> Result<Self, Error> {
        verify_approval(&hash, &approval)?;
        Ok(SignatureString { hash, approval })
    }

    /// The approval, if it is for `transaction`.
    pub fn approval_for(&self, transaction: &Transaction) -> Result<Approval, Error> {
        if self.hash != transaction.hash() {
            return Err(Error::OtherTransaction {
                expected: transaction.hash(),
                found: self.hash
            });
        }
        Ok(self.approval.clone())
    }
}

fn hash_hex(hash: &TransactionHash) -> (&'static str, String) {
    match hash {
        TransactionHash::V1(hash) => ("v1", format!("{:?}", hash.inner())),
        TransactionHash::Deploy(hash) => ("deploy", format!("{:?}", hash.inner()))
    }
}

impl fmt::Display for SignatureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, hash) = hash_hex(&self.hash);
        write!(
            f,
            "{PREFIX}:{kind}:{hash}:{}:{}",
            self.approval.signer().to_hex(),
            self.approval.signature().to_hex()
        )
    }
}

impl FromStr for SignatureString {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bad = |reason: &str| Error::BadSignatureString(reason.to_string());
        let parts: Vec<&str> = input.trim().split(':').collect();
        let [prefix, kind, hash, signer, signature] = parts[..] else {
            return Err(bad("it does not have five parts separated by ':'"));
        };
        if prefix != PREFIX {
            return Err(bad("it does not start with odra-sig"));
        }
        let digest = Digest::from_hex(hash)
            .map_err(|_| bad("the transaction hash is not 32 bytes of hex"))?;
        let hash = match kind {
            "v1" => TransactionHash::V1(TransactionV1Hash::new(digest)),
            "deploy" => TransactionHash::Deploy(DeployHash::new(digest)),
            _ => return Err(bad("the kind is neither v1 nor deploy"))
        };
        let signer = PublicKey::from_hex(signer).map_err(|_| bad("the public key is not valid"))?;
        if signer.is_system() {
            return Err(bad("the public key is the system key, which cannot sign"));
        }
        let signature =
            Signature::from_hex(signature).map_err(|_| bad("the signature is not valid hex"))?;
        SignatureString::new(hash, Approval::new(signer, signature))
    }
}
