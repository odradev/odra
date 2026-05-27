#![allow(missing_docs)]

//! CEP-2612 — an adaptation of [ERC-2612] for the Casper Network.
//!
//! ERC-2612 extends ERC-20 with a `permit` entry point that lets a token holder
//! grant an allowance via an off-chain signature instead of an on-chain
//! `approve` transaction. A third party (a relayer) can then submit the signed
//! permit on-chain, which makes the approval gas-less from the holder's
//! perspective and removes the classic "approve + transferFrom" two-transaction
//! pattern.
//!
//! # Differences from EVM ERC-2612
//!
//! The EIP-712 typed-data digest construction (domain separator + `Permit`
//! struct hash) follows the EVM specification exactly so signatures produced
//! by standard EIP-712 tooling remain compatible. The verification path
//! differs because Casper does not provide `ecrecover`:
//!
//! * The caller must pass the signer's [`PublicKey`] explicitly alongside the
//!   signature. The contract checks that `Address::from(public_key) == owner`
//!   before verifying the signature, so a valid signature from a different
//!   keypair cannot be used to approve on someone else's behalf.
//! * Signature verification uses the host's [`verify_signature`] facility,
//!   which supports Casper's Ed25519 and Secp256k1 account keys.
//! * The EIP-712 `chainId` field (a `uint256` on Ethereum) is replaced by a
//!   `chain_name` string (e.g. `"casper:casper"`) supplied at construction
//!   time, matching Casper's chain identification model.
//!
//! Replay protection follows the ERC-2612 pattern: each `owner` has a
//! monotonically increasing `nonce` that is mixed into the signed digest and
//! incremented on every successful `permit` call.
//!
//! [ERC-2612]: https://eips.ethereum.org/EIPS/eip-2612
//! [`verify_signature`]: odra::ContractEnv::verify_signature
use casper_eip_712::DomainSeparator;
use odra::{
    casper_types::{bytesrepr::Bytes, PublicKey, U256},
    named_keys::{base64_encoded_key_value_storage, single_value_storage},
    prelude::*
};

use crate::{cep18_token::Cep18, eip712};

// keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
const PERMIT_TYPEHASH: [u8; 32] = [
    0x6e, 0x71, 0xed, 0xae, 0x12, 0xb1, 0xb9, 0x7f, 0x4d, 0x1f, 0x60, 0x37, 0x0f, 0xef, 0x10, 0x10,
    0x5f, 0xa2, 0xfa, 0xae, 0x01, 0x26, 0x11, 0x4a, 0x16, 0x9c, 0x64, 0x84, 0x5d, 0x61, 0x26, 0xc9
];

/// Errors raised by CEP-2612 permit operations.
#[odra::odra_error]
pub enum Error {
    /// Signature verification against the supplied public key failed, or the
    /// recomputed digest does not match what the signer signed (e.g. wrong
    /// `value`, `nonce`, `spender`, or `deadline`).
    InvalidSignature = 36_000,
    /// The current block time is past the `deadline` timestamp. Permits with
    /// `deadline == u64::MAX` skip this check and never expire.
    PermitExpired = 36_001,
    /// The supplied `public_key` does not hash to the declared `owner`
    /// address. Guards against using a valid signature from a different
    /// keypair to approve on someone else's behalf.
    InvalidPublicKey = 36_002
}

/// Storage defined as named keys.
const CHAIN_NAME_KEY: &str = "chain_name";
const PERMIT_NONCES_KEY: &str = "permit_nonces";

/// Domain separator version.
const DOMAIN_VERSION: &str = "1";

single_value_storage!(
    CEP2612ChainNameStorage,
    String,
    CHAIN_NAME_KEY,
    ExecutionError::KeyNotFound
);

base64_encoded_key_value_storage!(CEP2612PermitNoncesStorage, PERMIT_NONCES_KEY, Address, U256);

/// CEP-2612 permit module — adds off-chain signed approvals to a CEP-18 token.
///
/// The module is meant to be composed with a [`Cep18`] sub-module (see
/// [`CEP2612Wrapper`] for a deployable composition used in tests).
#[odra::module]
pub struct CEP2612 {
    /// Per-owner monotonic nonce mixed into the signed digest to prevent
    /// replay of a previously consumed permit.
    permit_nonces: SubModule<CEP2612PermitNoncesStorage>,
    /// CAIP-2 chain name used as the EIP-712 domain's `chainId` substitute.
    chain_name: SubModule<CEP2612ChainNameStorage>,
    /// The CEP-18 token whose allowances are mutated by `permit`.
    token: SubModule<Cep18>
}

#[odra::module]
impl CEP2612 {
    /// Initializes the module by storing the EIP-712 domain's CAIP-2 chain name
    /// (e.g. `"casper:casper"`) and the nonce storage.
    pub fn init(&mut self, chain_name: String) {
        self.chain_name.set(chain_name);
        self.permit_nonces.init();
    }

    /// Consumes an off-chain permit signature and sets `spender`'s allowance
    /// over `owner`'s tokens to `value`.
    ///
    /// The caller (typically a relayer, not `owner`) supplies the signed
    /// `Permit(owner, spender, value, nonce, deadline)` typed-data digest.
    /// The contract:
    ///
    /// 1. Rejects the call if `deadline != u64::MAX` and the current block
    ///    time is past `deadline` (`PermitExpired`).
    /// 2. Rejects the call if `Address::from(public_key) != owner`
    ///    (`InvalidPublicKey`).
    /// 3. Recomputes the EIP-712 digest using the on-chain nonce for `owner`
    ///    and verifies `signature` against `public_key`. A mismatch — wrong
    ///    field value or a replayed signature whose nonce has already been
    ///    consumed — yields `InvalidSignature`.
    /// 4. Increments the owner's nonce and overwrites the allowance via
    ///    [`Cep18::raw_approve`] (it does not add to the existing allowance).
    pub fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: u64,
        public_key: PublicKey,
        signature: Bytes
    ) {
        if deadline != u64::MAX && self.env().get_block_time() > deadline {
            self.revert(Error::PermitExpired);
        }

        if Address::from(public_key.clone()) != owner {
            self.revert(Error::InvalidPublicKey);
        }

        let nonce = self.permit_nonces.get(&owner).unwrap_or_default();
        let message_hash = self.message_hash(owner, spender, value, nonce, deadline);
        let message = Bytes::from(message_hash.to_vec());

        let is_valid = self
            .env()
            .verify_signature(&message, &signature, &public_key);
        if !is_valid {
            self.revert(Error::InvalidSignature);
        }

        self.permit_nonces.set(&owner, nonce + U256::from(1));

        self.token.raw_approve(&owner, &spender, &value);
    }

    /// Builds the EIP-712 domain separator. Bound to the token `name`, the
    /// fixed `DOMAIN_VERSION`, the configured `chain_name`, and the
    /// deployed contract's own address (the EIP-712 `verifyingContract`).
    fn domain_separator(&self) -> DomainSeparator {
        let self_address = self.env().self_address();
        let name = self.token.name();
        let chain_name = self.chain_name.get();
        crate::eip712::domain_separator(&name, DOMAIN_VERSION, chain_name, self_address)
    }

    /// Computes the EIP-712 digest the signer must produce — the keccak256 of
    /// the encoded `Permit` struct combined with the domain separator. `value`
    /// and `nonce` are big-endian encoded to match the EVM `uint256` layout.
    fn message_hash(
        &self,
        owner: Address,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: u64
    ) -> [u8; 32] {
        let mut encoded_data = Vec::with_capacity(32 * 5);
        let mut value_bytes = [0u8; 32];
        value.to_big_endian(&mut value_bytes);
        let mut nonce_bytes = [0u8; 32];
        nonce.to_big_endian(&mut nonce_bytes);
        encoded_data.extend(eip712::encode_address(owner));
        encoded_data.extend(eip712::encode_address(spender));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint256(nonce_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(deadline));

        let domain = self.domain_separator();

        crate::eip712::hash_typed_data(domain, PERMIT_TYPEHASH, encoded_data)
    }
}

/// Wrapper contract that combines ERC-2612 functionality with a CEP-18 token for testing purposes.
#[odra::module]
pub struct CEP2612Wrapper {
    cep2612: SubModule<CEP2612>,
    token: SubModule<Cep18>
}

/// Wrapper contract that combines ERC-2612 functionality with a CEP-18 token for testing purposes.
/// In a real deployment, the ERC-2612 module would likely be separate and interact with an existing token contract.
#[odra::module]
impl CEP2612Wrapper {
    /// Initializes the wrapper by deploying the ERC-2612 module and the CEP-18 token, and setting up the EIP-712 domain.
    pub fn init(
        &mut self,
        chain_name: String,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256
    ) {
        self.cep2612.init(chain_name);
        self.token.init(symbol, name, decimals, initial_supply);
    }

    delegate! {
        to self.cep2612 {
            fn permit(
                &mut self,
                owner: Address,
                spender: Address,
                value: U256,
                deadline: u64,
                public_key: PublicKey,
                signature: Bytes
            );
        }

        to self.token {
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cep18::events::SetAllowance;
    use odra::host::{Deployer, HostEnv};

    const TOKEN_NAME: &str = "Test Token";
    const TOKEN_SYMBOL: &str = "TEST";
    const TOKEN_DECIMALS: u8 = 8;
    const INITIAL_SUPPLY: u64 = 1_000_000;
    const CHAIN_NAME: &str = "casper-test";

    struct Setup {
        env: HostEnv,
        wrapper: CEP2612WrapperHostRef,
        alice: Address,
        bob: Address,
        charlie: Address,
        alice_pubkey: PublicKey
    }

    fn setup() -> Setup {
        let env = odra_test::env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);
        let charlie = env.get_account(2);
        let alice_pubkey = env.public_key(&alice);

        let wrapper = CEP2612Wrapper::deploy(
            &env,
            CEP2612WrapperInitArgs {
                chain_name: CHAIN_NAME.to_string(),
                symbol: TOKEN_SYMBOL.to_string(),
                name: TOKEN_NAME.to_string(),
                decimals: TOKEN_DECIMALS,
                initial_supply: INITIAL_SUPPLY.into()
            }
        );

        Setup {
            env,
            wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        }
    }

    #[test]
    fn permit_happy_path() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        assert_eq!(wrapper.allowance(&alice, &bob), U256::zero());

        let value: U256 = 500u64.into();
        let deadline: u64 = u64::MAX;
        let nonce: U256 = U256::zero();

        let signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            value,
            nonce,
            deadline
        );

        // Charlie relays alice's permit (gas-less for alice).
        env.set_caller(charlie);
        wrapper.permit(alice, bob, value, deadline, alice_pubkey, signature);

        assert_eq!(wrapper.allowance(&alice, &bob), value);
        assert!(env.emitted_event(
            &wrapper,
            SetAllowance {
                owner: alice,
                spender: bob,
                allowance: value
            }
        ));
    }

    #[test]
    fn permit_replay_reverts() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let value: U256 = 500u64.into();
        let deadline: u64 = u64::MAX;
        let nonce: U256 = U256::zero();

        let signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            value,
            nonce,
            deadline
        );

        env.set_caller(charlie);
        wrapper.permit(
            alice,
            bob,
            value,
            deadline,
            alice_pubkey.clone(),
            signature.clone()
        );

        // Resubmitting fails: the on-chain nonce is now 1, so the contract
        // recomputes a different digest and the old signature no longer matches.
        assert_eq!(
            wrapper.try_permit(alice, bob, value, deadline, alice_pubkey, signature),
            Err(Error::InvalidSignature.into())
        );
    }

    #[test]
    fn permit_subsequent_with_next_nonce_succeeds() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let deadline: u64 = u64::MAX;
        let first_value: U256 = 500u64.into();
        let second_value: U256 = 1_000u64.into();

        let first_signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            first_value,
            U256::zero(),
            deadline
        );
        let second_signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            second_value,
            U256::one(),
            deadline
        );

        env.set_caller(charlie);
        wrapper.permit(
            alice,
            bob,
            first_value,
            deadline,
            alice_pubkey.clone(),
            first_signature
        );
        assert_eq!(wrapper.allowance(&alice, &bob), first_value);

        wrapper.permit(
            alice,
            bob,
            second_value,
            deadline,
            alice_pubkey,
            second_signature
        );
        // raw_approve overwrites (does not add) the allowance.
        assert_eq!(wrapper.allowance(&alice, &bob), second_value);
    }

    #[test]
    fn permit_expired() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        // Move block time past the chosen deadline.
        env.advance_block_time(60_000);

        let value: U256 = 500u64.into();
        let deadline: u64 = 1_000;
        let nonce: U256 = U256::zero();

        let signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            value,
            nonce,
            deadline
        );

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_permit(alice, bob, value, deadline, alice_pubkey, signature),
            Err(Error::PermitExpired.into())
        );
    }

    #[test]
    fn permit_max_deadline_skips_expiry_check() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        // Even after a long time, a permit with deadline = u64::MAX is valid.
        env.advance_block_time(1_000_000);

        let value: U256 = 500u64.into();
        let deadline: u64 = u64::MAX;
        let nonce: U256 = U256::zero();

        let signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            value,
            nonce,
            deadline
        );

        env.set_caller(charlie);
        wrapper.permit(alice, bob, value, deadline, alice_pubkey, signature);

        assert_eq!(wrapper.allowance(&alice, &bob), value);
    }

    #[test]
    fn permit_invalid_signature() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let deadline: u64 = u64::MAX;
        let nonce: U256 = U256::zero();

        // Alice signs for value=100 ...
        let signature = sign_permit(
            &env,
            &alice,
            wrapper.address(),
            alice,
            bob,
            U256::from(100u64),
            nonce,
            deadline
        );

        // ... but charlie submits with value=200, so the digest the contract
        // recomputes won't match the signature.
        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_permit(
                alice,
                bob,
                U256::from(200u64),
                deadline,
                alice_pubkey,
                signature
            ),
            Err(Error::InvalidSignature.into())
        );
    }

    #[test]
    fn permit_with_mismatched_public_key_reverts() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            ..
        } = setup();

        let attacker = env.get_account(3);
        let attacker_pubkey = env.public_key(&attacker);

        let value: U256 = 500u64.into();
        let deadline: u64 = u64::MAX;
        let nonce: U256 = U256::zero();

        // Attacker signs a permit message that names alice as the owner,
        // using their own keypair.
        let signature = sign_permit(
            &env,
            &attacker,
            wrapper.address(),
            alice,
            bob,
            value,
            nonce,
            deadline
        );

        // Submitting with owner=alice but public_key=attacker_pubkey must fail,
        // even though the signature itself verifies against the attacker's key.
        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_permit(alice, bob, value, deadline, attacker_pubkey, signature),
            Err(Error::InvalidPublicKey.into())
        );

        // Allowance must remain unchanged.
        assert_eq!(wrapper.allowance(&alice, &bob), U256::zero());
    }

    #[allow(clippy::too_many_arguments)]
    fn sign_permit(
        env: &HostEnv,
        signer: &Address,
        contract_address: Address,
        owner: Address,
        spender: Address,
        value: U256,
        nonce: U256,
        deadline: u64
    ) -> Bytes {
        let mut value_bytes = [0u8; 32];
        value.to_big_endian(&mut value_bytes);
        let mut nonce_bytes = [0u8; 32];
        nonce.to_big_endian(&mut nonce_bytes);

        let mut encoded_data = Vec::with_capacity(32 * 5);
        encoded_data.extend(crate::eip712::encode_address(owner));
        encoded_data.extend(crate::eip712::encode_address(spender));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint256(nonce_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(deadline));

        let domain = crate::eip712::domain_separator(
            TOKEN_NAME,
            DOMAIN_VERSION,
            CHAIN_NAME.to_string(),
            contract_address
        );
        let message_hash = crate::eip712::hash_typed_data(domain, PERMIT_TYPEHASH, encoded_data);
        let message = Bytes::from(message_hash.to_vec());

        env.sign_message(&message, signer)
    }
}
