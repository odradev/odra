//! Signatures made independently and merged, the transaction file, the signature string,
//! inspection against an account and the file transport.

use std::collections::{BTreeMap, BTreeSet};

use casper_types::{
    account::AccountHash,
    bytesrepr::{Bytes, ToBytes},
    Digest, InitiatorAddr, PricingMode, PublicKey, RuntimeArgs, SecretKey, TimeDiff, Timestamp,
    Transaction, TransactionArgs, TransactionEntryPoint, TransactionRuntimeParams,
    TransactionScheduling, TransactionTarget, TransactionV1, TransactionV1Payload
};
use odra_casper_account_manager::{
    AccountChange, AccountState, ThresholdKind, ACCOUNT_MANAGER_WASM
};
use odra_casper_signing::{
    decode_account_change, inspect, merge, read_transaction, sign, write_transaction, Error,
    FileTransport, SignatureString, SigningTransport, Status, MAX_APPROVALS
};

const NOW: u64 = 1_800_000_000_000;

fn ed25519(n: u8) -> SecretKey {
    SecretKey::ed25519_from_bytes([n; 32]).unwrap()
}

fn secp256k1(n: u8) -> SecretKey {
    SecretKey::secp256k1_from_bytes([n; 32]).unwrap()
}

fn hash_of(key: &SecretKey) -> AccountHash {
    PublicKey::from(key).to_account_hash()
}

/// An unsigned session transaction, built as casper-client 5.0.0's builder does.
fn session(wasm: &[u8], args: RuntimeArgs, account: AccountHash) -> Transaction {
    let target = TransactionTarget::Session {
        is_install_upgrade: false,
        module_bytes: Bytes::from(wasm.to_vec()),
        runtime: TransactionRuntimeParams::VmCasperV1
    };
    let mut fields = BTreeMap::new();
    fields.insert(
        0u16,
        TransactionArgs::Named(args).to_bytes().unwrap().into()
    );
    fields.insert(1u16, target.to_bytes().unwrap().into());
    fields.insert(2u16, TransactionEntryPoint::Call.to_bytes().unwrap().into());
    fields.insert(
        3u16,
        TransactionScheduling::Standard.to_bytes().unwrap().into()
    );
    let payload = TransactionV1Payload::new(
        "casper-test".to_string(),
        Timestamp::from(NOW),
        TimeDiff::from_seconds(7200),
        PricingMode::PaymentLimited {
            payment_amount: 2_500_000_000,
            gas_price_tolerance: 1,
            standard_payment: true
        },
        InitiatorAddr::AccountHash(account),
        fields
    );
    let hash = Digest::hash(payload.to_bytes().unwrap());
    Transaction::V1(TransactionV1::new(hash.into(), payload, BTreeSet::new()))
}

/// A 2-of-3 multisig account: keys 1, 2 (ed25519) and 3 (secp256k1) with weight 1, deployment
/// and key_management 2.
fn account() -> (AccountState, [SecretKey; 3]) {
    let keys = [ed25519(1), ed25519(2), secp256k1(3)];
    let mut state = AccountState::new(hash_of(&keys[0]));
    state.associated_keys.insert(hash_of(&keys[1]), 1);
    state.associated_keys.insert(hash_of(&keys[2]), 1);
    state.deployment = 2;
    state.key_management = 2;
    (state, keys)
}

fn call(account: AccountHash) -> Transaction {
    session(
        b"\0asm not the account manager",
        RuntimeArgs::new(),
        account
    )
}

#[test]
fn independent_signatures_merge_into_a_valid_transaction() {
    let (state, keys) = account();
    let unsigned = call(hash_of(&keys[0]));
    let hash = unsigned.hash();

    // Each signer signs a copy of the same file on their own machine.
    let first = sign(&unsigned, &keys[0]);
    let third = sign(&unsigned, &keys[2]);
    let merged = merge(unsigned.clone(), [first.clone(), third]).unwrap();
    assert_eq!(merged.hash(), hash);
    assert_eq!(merged.approvals().len(), 2);
    merged.verify().expect("hash and both signatures verify");

    // Merging the same approval again changes nothing.
    let again = merge(merged.clone(), [first]).unwrap();
    assert_eq!(again, merged);

    let inspection = inspect(&merged, &state, Timestamp::from(NOW + 1000));
    assert_eq!(inspection.status, Status::Ready);
    assert_eq!(inspection.weight, 2);
    assert_eq!(inspection.required, ThresholdKind::Deployment);
    assert_eq!(inspection.account, hash_of(&keys[0]));
}

#[test]
fn rejects_a_signature_of_another_transaction() {
    let (_, keys) = account();
    let one = call(hash_of(&keys[0]));
    let other = session(b"\0asm other", RuntimeArgs::new(), hash_of(&keys[0]));
    let wrong = sign(&other, &keys[1]);
    assert!(matches!(
        merge(one.clone(), [wrong.clone()]),
        Err(Error::BadSignature(_))
    ));

    let line = SignatureString::new(other.hash(), wrong).unwrap();
    assert!(matches!(
        line.approval_for(&one),
        Err(Error::OtherTransaction { .. })
    ));
}

#[test]
fn rejects_a_forged_signature() {
    let (_, keys) = account();
    let unsigned = call(hash_of(&keys[0]));
    let real = sign(&unsigned, &keys[1]);
    // Key 2's signature presented as key 1's.
    let forged = casper_types::Approval::new(PublicKey::from(&keys[0]), *real.signature());
    assert!(matches!(
        merge(unsigned, [forged]),
        Err(Error::BadSignature(_))
    ));
}

#[test]
fn rejects_too_many_approvals() {
    let unsigned = call(AccountHash::new([7; 32]));
    let approvals: Vec<_> = (1..=MAX_APPROVALS as u8 + 1)
        .map(|n| sign(&unsigned, &ed25519(n)))
        .collect();
    assert!(matches!(
        merge(unsigned, approvals),
        Err(Error::TooManyApprovals(101))
    ));
}

#[test]
fn the_signature_string_round_trips() {
    let (_, keys) = account();
    let unsigned = call(hash_of(&keys[0]));
    for key in &keys {
        let line = SignatureString::new(unsigned.hash(), sign(&unsigned, key)).unwrap();
        let text = line.to_string();
        assert!(text.starts_with("odra-sig:v1:"), "{text}");
        let parsed: SignatureString = text.parse().unwrap();
        assert_eq!(parsed, line);
        assert_eq!(parsed.approval_for(&unsigned).unwrap(), line.approval);
    }
    for bad in [
        "",
        "odra-sig:v1:00",
        "sig:v1:aa:bb:cc",
        "odra-sig:v2:00:00:00",
        &format!("odra-sig:v1:{}:00:00", "a".repeat(64))
    ] {
        assert!(
            matches!(
                bad.parse::<SignatureString>(),
                Err(Error::BadSignatureString(_))
            ),
            "{bad}"
        );
    }
    // Well formed, but the signature is not of this hash.
    let line = SignatureString::new(unsigned.hash(), sign(&unsigned, &keys[0]))
        .unwrap()
        .to_string();
    let tampered = line.replacen("odra-sig:v1:", &format!("odra-sig:v1:{}", "0"), 1);
    assert!(tampered.parse::<SignatureString>().is_err());
}

#[test]
fn inspection_reports_what_is_missing() {
    let (state, keys) = account();
    let unsigned = call(hash_of(&keys[0]));
    let now = Timestamp::from(NOW + 1000);

    assert_eq!(
        inspect(&unsigned, &state, now).status,
        Status::Pending { missing: 2 }
    );
    let one = merge(unsigned.clone(), [sign(&unsigned, &keys[1])]).unwrap();
    let inspection = inspect(&one, &state, now);
    assert_eq!(inspection.status, Status::Pending { missing: 1 });
    assert_eq!(inspection.signers[0].weight, Some(1));

    // A key that is not associated makes the whole transaction invalid.
    let stranger = merge(one.clone(), [sign(&one, &ed25519(9))]).unwrap();
    let inspection = inspect(&stranger, &state, now);
    assert_eq!(inspection.status, Status::ForeignSigner);
    assert!(inspection
        .signers
        .iter()
        .any(|signer| signer.weight.is_none()));

    // After timestamp + ttl it is over.
    let late = Timestamp::from(NOW + 7_200_001);
    assert_eq!(inspect(&one, &state, late).status, Status::Expired);
}

#[test]
fn an_account_manager_transaction_needs_key_management() {
    let (mut state, keys) = account();
    state.key_management = 3;
    let change = AccountChange::new()
        .set_key(AccountHash::new([5; 32]), 1)
        .deployment(1);
    let unsigned = session(
        ACCOUNT_MANAGER_WASM,
        change.to_runtime_args(),
        hash_of(&keys[0])
    );
    assert_eq!(decode_account_change(&unsigned), Some(change));

    let two = merge(
        unsigned.clone(),
        [sign(&unsigned, &keys[0]), sign(&unsigned, &keys[1])]
    )
    .unwrap();
    let inspection = inspect(&two, &state, Timestamp::from(NOW));
    assert_eq!(inspection.required, ThresholdKind::KeyManagement);
    assert_eq!(inspection.status, Status::Pending { missing: 1 });

    // Another wasm with the same arguments is not taken for the account manager.
    let lookalike = session(
        b"\0asm",
        AccountChange::new().deployment(1).to_runtime_args(),
        hash_of(&keys[0])
    );
    assert_eq!(decode_account_change(&lookalike), None);
}

#[test]
fn the_file_keeps_the_transaction_and_its_signatures() {
    let (_, keys) = account();
    let dir = std::env::temp_dir().join(format!("odra-signing-file-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tx.json");

    let unsigned = call(hash_of(&keys[0]));
    write_transaction(&path, &unsigned).unwrap();
    let read = read_transaction(&path).unwrap();
    assert_eq!(read, unsigned);
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(json.get("Version1").is_some());

    let signed = merge(read, [sign(&unsigned, &keys[0])]).unwrap();
    write_transaction(&path, &signed).unwrap();
    assert_eq!(read_transaction(&path).unwrap().approvals().len(), 1);

    // A file whose content was changed no longer matches its hash.
    let tampered = std::fs::read_to_string(&path)
        .unwrap()
        .replace("casper-test", "casper-tess");
    std::fs::write(&path, tampered).unwrap();
    assert!(matches!(read_transaction(&path), Err(Error::Invalid(_))));
    assert!(matches!(
        read_transaction(&dir.join("missing.json")),
        Err(Error::Io { .. })
    ));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_file_transport_collects_approvals() {
    let (state, keys) = account();
    let dir = std::env::temp_dir().join(format!("odra-signing-transport-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let transport = FileTransport::new(&dir);

    let unsigned = call(hash_of(&keys[0]));
    let hash = unsigned.hash();
    assert!(matches!(
        transport.fetch(&hash),
        Err(Error::UnknownTransaction(_))
    ));
    let first = merge(unsigned.clone(), [sign(&unsigned, &keys[0])]).unwrap();
    transport.publish(&first).unwrap();

    // Another signer fetches it, signs and adds the approval.
    let fetched = transport.fetch(&hash).unwrap();
    transport
        .add_approval(&hash, sign(&fetched, &keys[2]))
        .unwrap();
    let collected = transport.fetch(&hash).unwrap();
    assert_eq!(collected.approvals().len(), 2);
    assert_eq!(
        inspect(&collected, &state, Timestamp::from(NOW)).status,
        Status::Ready
    );

    // Publishing the first version again does not lose the second approval.
    transport.publish(&first).unwrap();
    assert_eq!(transport.fetch(&hash).unwrap().approvals().len(), 2);

    // A signature of another transaction is refused.
    let other = session(b"\0asm other", RuntimeArgs::new(), hash_of(&keys[0]));
    assert!(transport
        .add_approval(&hash, sign(&other, &keys[1]))
        .is_err());
    assert!(transport.path(&hash).starts_with(transport.dir()));
    std::fs::remove_dir_all(dir).unwrap();
}
