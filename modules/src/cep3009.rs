#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)]
//! CEP-3009 — an adaptation of [ERC-3009] for the Casper Network.
//!
//! ERC-3009 ("Transfer With Authorization") lets a token holder authorize a
//! transfer of their tokens via an off-chain EIP-712 signature. A third party
//! (a relayer, or the recipient themselves) submits the authorization
//! on-chain, so the token holder never has to spend gas to move their tokens.
//! Unlike ERC-2612, no allowance is granted: each authorization is a direct,
//! single-use mandate to move a specific `amount` to a specific `to` address.
//!
//! The module exposes three entry points:
//!
//! * [`CEP3009::transfer_with_authorization`] — anyone may relay; the funds
//!   move from `from` to `to` once the signature checks out.
//! * [`CEP3009::receive_with_authorization`] — only `to` may submit. On
//!   Ethereum this exists to prevent front-running of a relayed
//!   `to == contract` authorization; Casper has no public mempool, so
//!   front-running isn't a concern. The variant is still useful because it
//!   guarantees `caller == to`, which lets a receiving contract atomically
//!   accept the transfer and act on the deposit in the same call.
//! * [`CEP3009::cancel_authorization`] — the authorizer (or anyone relaying
//!   the authorizer's signed cancel message) can burn an unused nonce so a
//!   leaked authorization can never be redeemed.
//!
//! # Differences from EVM ERC-3009
//!
//! The EIP-712 typed-data digest construction (domain separator + the
//! `TransferWithAuthorization`, `ReceiveWithAuthorization`, and
//! `CancelAuthorization` struct hashes) follows the EVM specification exactly,
//! so signatures produced by standard EIP-712 tooling remain compatible. The
//! verification path differs because Casper does not provide `ecrecover`:
//!
//! * The caller must pass the signer's [`PublicKey`] explicitly alongside the
//!   signature. The contract checks that `Address::from(public_key) == from`
//!   (or `authorizer`, for cancellations) before verifying the signature, so
//!   a valid signature from a different keypair cannot be used to move
//!   someone else's tokens or burn someone else's nonce.
//! * Signature verification uses the host's [`verify_signature`] facility,
//!   which supports Casper's Ed25519 and Secp256k1 account keys.
//! * The EIP-712 `chainId` field (a `uint256` on Ethereum) is replaced by a
//!   `chain_name` string supplied at construction time, matching Casper's
//!   chain identification model.
//!
//! Replay protection follows ERC-3009: each authorization carries a 32-byte
//! `nonce` chosen by the signer (not a monotonic counter as in ERC-2612), and
//! the contract records `(authorizer, nonce) -> used` so the same nonce can
//! never be redeemed twice. Cancellation marks a nonce used without moving
//! funds.
//!
//! [ERC-3009]: https://eips.ethereum.org/EIPS/eip-3009
//! [`verify_signature`]: odra::ContractEnv::verify_signature

use crate::{cep18::events::Transfer, cep18_token::Cep18, eip712};
use casper_eip_712::DomainSeparator;
use odra::{
    casper_types::{bytesrepr::Bytes, PublicKey, U256},
    named_keys::{compound_key_value_storage, single_value_storage},
    prelude::*
};

// keccak256("TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)")
const TRANSFER_WITH_AUTHORIZATION_TYPEHASH: [u8; 32] = [
    0x7c, 0x7c, 0x6c, 0xdb, 0x67, 0xa1, 0x87, 0x43, 0xf4, 0x9e, 0xc6, 0xfa, 0x9b, 0x35, 0xf5, 0x0d,
    0x52, 0xed, 0x05, 0xcb, 0xed, 0x4c, 0xc5, 0x92, 0xe1, 0x3b, 0x44, 0x50, 0x1c, 0x1a, 0x22, 0x67
];

// keccak256("ReceiveWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)")
const RECEIVE_WITH_AUTHORIZATION_TYPEHASH: [u8; 32] = [
    0xd0, 0x99, 0xcc, 0x98, 0xef, 0x71, 0x10, 0x7a, 0x61, 0x6c, 0x4f, 0x0f, 0x94, 0x1f, 0x04, 0xc3,
    0x22, 0xd8, 0xe2, 0x54, 0xfe, 0x26, 0xb3, 0xc6, 0x66, 0x8d, 0xb8, 0x7a, 0xae, 0x41, 0x3d, 0xe8
];

// keccak256("CancelAuthorization(address authorizer,bytes32 nonce)")
const CANCEL_AUTHORIZATION_TYPEHASH: [u8; 32] = [
    0x15, 0x8b, 0x0a, 0x9e, 0xdf, 0x7a, 0x82, 0x8a, 0xad, 0x02, 0xf6, 0x3c, 0xd5, 0x15, 0xc6, 0x8e,
    0xf2, 0xf5, 0x0b, 0xa8, 0x07, 0x39, 0x6f, 0x6d, 0x12, 0x84, 0x28, 0x33, 0xa1, 0x59, 0x74, 0x29
];

/// Emitted when an authorization is consumed by
/// [`CEP3009::transfer_with_authorization`] or
/// [`CEP3009::receive_with_authorization`]. The `(authorizer, nonce)` pair
/// is now marked used and cannot be redeemed again.
#[odra::event]
pub struct AuthorizationUsed {
    authorizer: Address,
    nonce: Bytes
}

/// Emitted when an authorization nonce is burned by
/// [`CEP3009::cancel_authorization`] before being used. The `(authorizer,
/// nonce)` pair is marked used without any funds moving.
#[odra::event]
pub struct AuthorizationCanceled {
    authorizer: Address,
    nonce: Bytes
}

/// Errors raised by CEP-3009 authorization operations.
#[odra::odra_error]
pub enum Error {
    /// The `(from, nonce)` pair has already been consumed by a transfer or
    /// burned by a cancel. Authorizations are single-use.
    NonceAlreadyUsed = 37_000,
    /// The current block time is past `valid_before` — the authorization has
    /// expired.
    AuthorizationExpired = 37_001,
    /// The current block time is at or before `valid_after` — the
    /// authorization is not yet in its validity window.
    AuthorizationNotYetValid = 37_002,
    /// Signature verification against the supplied public key failed, or the
    /// recomputed digest does not match what the signer signed (e.g. wrong
    /// `amount`, `to`, or `nonce`).
    InvalidSignature = 37_003,
    /// The supplied `public_key` does not hash to the declared `from` /
    /// `authorizer` address. Guards against using a valid signature from a
    /// different keypair to move someone else's tokens or burn their nonce.
    InvalidPublicKey = 37_004,
    /// `receive_with_authorization` was submitted by an account other than
    /// `to`. See the module-level docs for why this restriction exists.
    InvalidCaller = 37_005,
    /// Attempted to cancel an authorization whose `(authorizer, nonce)` pair
    /// has already been consumed.
    AuthorizationUsed = 37_006,
    /// The supplied `nonce` is not exactly 32 bytes. ERC-3009 nonces are
    /// `bytes32`; the EIP-712 digest only covers 32 bytes, so accepting a
    /// shorter or longer nonce would let an attacker craft length aliases
    /// that share a digest (and thus a signature) while occupying distinct
    /// replay-protection storage keys.
    InvalidNonceLength = 37_007
}

/// Storage defined as named keys.
const CHAIN_NAME_KEY: &str = "chain_name";
const USED_NONCES_KEY: &str = "used_nonces";

/// Domain separator version.
const DOMAIN_VERSION: &str = "1";

single_value_storage!(
    CEP3009ChainNameStorage,
    String,
    CHAIN_NAME_KEY,
    ExecutionError::KeyNotFound
);

compound_key_value_storage!(
    CEP3009UsedNoncesStorage,
    USED_NONCES_KEY,
    Address,
    Bytes,
    bool
);

/// CEP-3009 authorization module — adds off-chain signed transfers to a
/// CEP-18 token.
///
/// The module is meant to be composed with a [`Cep18`] sub-module (see
/// [`CEP3009Wrapper`] for a deployable composition used in tests).
#[odra::module(events = [AuthorizationUsed, AuthorizationCanceled, Transfer], errors = Error)]
pub struct CEP3009 {
    /// Per-(authorizer, nonce) flag recording whether that authorization has
    /// been consumed or cancelled. Provides single-use semantics.
    used_nonces: SubModule<CEP3009UsedNoncesStorage>,
    /// CAIP-2 Chain name used as the EIP-712 domain's `chainId` substitute.
    chain_name: SubModule<CEP3009ChainNameStorage>,
    /// The CEP-18 token whose balances are mutated by authorized transfers.
    token: SubModule<Cep18>
}

#[odra::module]
impl CEP3009 {
    /// Initializes the module by storing the EIP-712 chain ID and the
    /// used-nonces storage.
    pub fn init(&mut self, chain_name: String) {
        self.chain_name.set(chain_name);
        self.used_nonces.init();
    }

    /// Returns `true` if `(authorizer, nonce)` has been consumed by a
    /// previous transfer or burned by [`Self::cancel_authorization`]. Useful
    /// for off-chain clients to check before submitting a relayed transfer.
    pub fn authorization_state(&self, authorizer: Address, nonce: Bytes) -> bool {
        self.used_nonces.get_or_default(&authorizer, &nonce)
    }

    /// Consumes a signed `TransferWithAuthorization` and moves `amount`
    /// tokens from `from` to `to`. Any account may submit this call — the
    /// signature, not the caller, is what authorizes the transfer.
    ///
    /// Reverts with [`Error::InvalidNonceLength`], [`Error::NonceAlreadyUsed`],
    /// [`Error::AuthorizationNotYetValid`], [`Error::AuthorizationExpired`],
    /// [`Error::InvalidPublicKey`], or [`Error::InvalidSignature`] if the
    /// corresponding precondition fails.
    /// On success, marks the nonce used, emits [`AuthorizationUsed`], and
    /// performs the transfer via [`Cep18::raw_transfer`].
    pub fn transfer_with_authorization(
        &mut self,
        from: Address,
        to: Address,
        amount: U256,
        valid_after: u64,
        valid_before: u64,
        nonce: Bytes,
        public_key: PublicKey,
        signature: Bytes
    ) {
        self.raw_transfer_with_authorization(
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            from,
            to,
            amount,
            valid_after,
            valid_before,
            nonce,
            public_key,
            signature
        );
    }

    /// Like [`Self::transfer_with_authorization`], but requires
    /// `caller == to`. This lets a receiving contract atomically accept a
    /// pre-signed transfer and act on it within the same call (e.g. a
    /// "deposit then mint LP token" flow), while preventing any other
    /// account from triggering the transfer at a moment of their choosing.
    ///
    /// Reverts with [`Error::InvalidCaller`] if `caller != to`. All other
    /// checks and effects mirror `transfer_with_authorization`.
    pub fn receive_with_authorization(
        &mut self,
        from: Address,
        to: Address,
        amount: U256,
        valid_after: u64,
        valid_before: u64,
        nonce: Bytes,
        public_key: PublicKey,
        signature: Bytes
    ) {
        let caller = self.env().caller();
        if caller != to {
            self.env().revert(Error::InvalidCaller);
        }

        self.raw_transfer_with_authorization(
            RECEIVE_WITH_AUTHORIZATION_TYPEHASH,
            from,
            to,
            amount,
            valid_after,
            valid_before,
            nonce,
            public_key,
            signature
        );
    }

    /// Burns an unused authorization nonce by consuming a signed
    /// `CancelAuthorization` message. Once a nonce is cancelled, any
    /// previously distributed signature for the same `(authorizer, nonce)`
    /// pair becomes unredeemable — the canonical way for an authorizer to
    /// revoke a leaked or otherwise unwanted authorization.
    ///
    /// Reverts with [`Error::InvalidNonceLength`] if `nonce` is not 32 bytes,
    /// with [`Error::AuthorizationUsed`] if the nonce has already been consumed
    /// (a cancelled nonce cannot be re-cancelled either), with
    /// [`Error::InvalidPublicKey`] if `public_key` does not hash to
    /// `authorizer`, or with [`Error::InvalidSignature`] if signature
    /// verification fails. On success, marks the nonce used and emits
    /// [`AuthorizationCanceled`].
    pub fn cancel_authorization(
        &mut self,
        authorizer: Address,
        nonce: Bytes,
        public_key: PublicKey,
        signature: Bytes
    ) {
        let Ok(nonce_bytes) = <[u8; 32]>::try_from(nonce.as_slice()) else {
            self.env().revert(Error::InvalidNonceLength)
        };

        if self.authorization_state(authorizer, nonce.clone()) {
            self.env().revert(Error::AuthorizationUsed);
        }

        if Address::from(public_key.clone()) != authorizer {
            self.env().revert(Error::InvalidPublicKey);
        }

        let message = self.build_cancel_message(authorizer, nonce_bytes);
        if !self
            .env()
            .verify_signature(&message, &signature, &public_key)
        {
            self.env().revert(Error::InvalidSignature);
        }

        self.used_nonces.set(&authorizer, &nonce, true);
        self.env()
            .emit_event(AuthorizationCanceled { authorizer, nonce });
    }
}

impl CEP3009 {
    /// Shared body of [`Self::transfer_with_authorization`] and
    /// [`Self::receive_with_authorization`]. They differ only in the EIP-712
    /// typehash used to compute the digest (and in the caller restriction
    /// applied by the receive variant before calling this).
    fn raw_transfer_with_authorization(
        &mut self,
        typehash: [u8; 32],
        from: Address,
        to: Address,
        amount: U256,
        valid_after: u64,
        valid_before: u64,
        nonce: Bytes,
        public_key: PublicKey,
        signature: Bytes
    ) {
        // 1. Enforce a 32-byte nonce so the replay-protection key and the
        // signed digest cover the exact same bytes (no length aliasing).
        let Ok(nonce_bytes) = <[u8; 32]>::try_from(nonce.as_slice()) else {
            self.env().revert(Error::InvalidNonceLength)
        };

        // 2. Replay protection
        if self.used_nonces.get_or_default(&from, &nonce) {
            self.env().revert(Error::NonceAlreadyUsed);
        }

        // 3. block_time
        let now_secs = self.env().get_block_time_secs();

        // 4. Check valid_after
        if now_secs <= valid_after {
            self.env().revert(Error::AuthorizationNotYetValid);
        }

        // 5. Check valid_before
        if now_secs >= valid_before {
            self.env().revert(Error::AuthorizationExpired);
        }

        // 6. Verify that public_key matches the `from` address
        let derived_address = Address::from(public_key.clone());
        if derived_address != from {
            self.env().revert(Error::InvalidPublicKey);
        }

        // 7. Build message and verify signature
        let message = self.build_authorization_message(
            typehash,
            from,
            to,
            &amount,
            valid_after,
            valid_before,
            nonce_bytes
        );

        if !self
            .env()
            .verify_signature(&message, &signature, &public_key)
        {
            self.env().revert(Error::InvalidSignature);
        }

        // 8. Mark nonce as used
        self.used_nonces.set(&from, &nonce, true);

        // 9. Emit event
        self.env().emit_event(AuthorizationUsed {
            authorizer: from,
            nonce
        });

        // 10. Execute transfer (raw_transfer takes refs)
        self.token.raw_transfer(&from, &to, &amount);

        // 11. Emit event.
        self.env().emit_event(Transfer {
            sender: from,
            recipient: to,
            amount
        });
    }

    /// Builds the EIP-712 digest for a transfer authorization. `amount` is
    /// big-endian encoded to match the EVM `uint256` layout, and `nonce` is the
    /// 32-byte `bytes32` value (enforced at every entry point).
    fn build_authorization_message(
        &self,
        typehash: [u8; 32],
        from: Address,
        to: Address,
        amount: &U256,
        valid_after: u64,
        valid_before: u64,
        nonce: [u8; 32]
    ) -> Bytes {
        let mut value_bytes = [0u8; 32];
        amount.to_big_endian(&mut value_bytes);

        let mut encoded_data = Vec::with_capacity(6 * 32);
        encoded_data.extend(eip712::encode_address(from));
        encoded_data.extend(eip712::encode_address(to));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_after));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_before));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce));
        let domain = self.domain_separator();

        Bytes::from(crate::eip712::hash_typed_data(domain, typehash, encoded_data).to_vec())
    }

    /// Builds the EIP-712 digest for a `CancelAuthorization` message. `nonce` is
    /// the 32-byte `bytes32` value (enforced at every entry point).
    fn build_cancel_message(&self, authorizer: Address, nonce: [u8; 32]) -> Bytes {
        let mut encoded_data = Vec::with_capacity(64);
        encoded_data.extend(eip712::encode_address(authorizer));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce));
        let domain = self.domain_separator();

        Bytes::from(
            eip712::hash_typed_data(domain, CANCEL_AUTHORIZATION_TYPEHASH, encoded_data).to_vec()
        )
    }

    /// Builds the EIP-712 domain separator. Bound to the token `name`, the
    /// fixed `DOMAIN_VERSION`, the configured `chain_name`, and the
    /// deployed contract's own address (the EIP-712 `verifyingContract`).
    fn domain_separator(&self) -> DomainSeparator {
        let self_address = self.env().self_address();
        let name = self.token.name();
        let chain_name = self.chain_name.get();
        eip712::domain_separator(&name, DOMAIN_VERSION, chain_name, self_address)
    }
}

/// Wrapper contract that combines ERC-3009 functionality with a CEP-18 token for testing purposes.
#[odra::module]
pub struct CEP3009Wrapper {
    cep3009: SubModule<CEP3009>,
    token: SubModule<Cep18>
}

#[odra::module]
impl CEP3009Wrapper {
    /// Initializes both the authorization module (storing the EIP-712 chain
    /// id) and the underlying CEP-18 token (symbol, name, decimals, supply).
    pub fn init(
        &mut self,
        chain_name: String,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256
    ) {
        self.cep3009.init(chain_name);
        self.token.init(symbol, name, decimals, initial_supply);
    }

    delegate! {
        to self.cep3009 {
            fn authorization_state(&self, authorizer: Address, nonce: Bytes) -> bool;
            fn transfer_with_authorization(
                &mut self,
                from: Address,
                to: Address,
                amount: U256,
                valid_after: u64,
                valid_before: u64,
                nonce: Bytes,
                public_key: PublicKey,
                signature: Bytes
            );
            fn receive_with_authorization(
                &mut self,
                from: Address,
                to: Address,
                amount: U256,
                valid_after: u64,
                valid_before: u64,
                nonce: Bytes,
                public_key: PublicKey,
                signature: Bytes
            );
            fn cancel_authorization(
                &mut self,
                authorizer: Address,
                nonce: Bytes,
                public_key: PublicKey,
                signature: Bytes
            );
        }

        to self.token {
            fn balance_of(&self, address: &Address) -> U256;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostEnv};

    const TOKEN_NAME: &str = "Test Token";
    const TOKEN_SYMBOL: &str = "TEST";
    const TOKEN_DECIMALS: u8 = 8;
    const INITIAL_SUPPLY: u64 = 1_000_000;
    const CHAIN_NAME: &str = "casper:casper";

    struct Setup {
        env: HostEnv,
        wrapper: CEP3009WrapperHostRef,
        alice: Address,
        bob: Address,
        charlie: Address,
        alice_pubkey: PublicKey
    }

    /// Deploys the wrapper with alice as deployer (so she gets the initial supply)
    /// and advances block time past `valid_after = 0`.
    fn setup() -> Setup {
        let env = odra_test::env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);
        let charlie = env.get_account(2);
        let alice_pubkey = env.public_key(&alice);

        let wrapper = CEP3009Wrapper::deploy(
            &env,
            CEP3009WrapperInitArgs {
                chain_name: CHAIN_NAME.to_string(),
                symbol: TOKEN_SYMBOL.to_string(),
                name: TOKEN_NAME.to_string(),
                decimals: TOKEN_DECIMALS,
                initial_supply: INITIAL_SUPPLY.into()
            }
        );

        env.advance_block_time(1_000);

        Setup {
            env,
            wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        }
    }

    fn fresh_nonce(seed: u8) -> Bytes {
        Bytes::from(vec![seed; 32])
    }

    #[test]
    fn transfer_with_authorization_happy_path() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        assert_eq!(wrapper.balance_of(&alice), INITIAL_SUPPLY.into());
        assert_eq!(wrapper.balance_of(&bob), U256::zero());

        let amount: U256 = 100u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(1);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        // Charlie relays the authorization on alice's behalf.
        env.set_caller(charlie);
        wrapper.transfer_with_authorization(
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            nonce.clone(),
            alice_pubkey,
            signature
        );

        assert_eq!(
            wrapper.balance_of(&alice),
            U256::from(INITIAL_SUPPLY) - amount
        );
        assert_eq!(wrapper.balance_of(&bob), amount);
        assert!(wrapper.authorization_state(alice, nonce.clone()));
        assert!(env.emitted_event(
            &wrapper,
            AuthorizationUsed {
                authorizer: alice,
                nonce
            }
        ));
    }

    #[test]
    fn transfer_with_authorization_replay_protection() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let amount: U256 = 100u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(2);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        env.set_caller(charlie);
        wrapper.transfer_with_authorization(
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            nonce.clone(),
            alice_pubkey.clone(),
            signature.clone()
        );

        // Submitting the same authorization a second time reverts.
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::NonceAlreadyUsed.into())
        );
    }

    #[test]
    fn transfer_with_authorization_not_yet_valid() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        // Block time is at 1 second; require it to be in the future.
        let valid_after: u64 = 60;
        let valid_before: u64 = u64::MAX;
        let amount: U256 = 100u64.into();
        let nonce = fresh_nonce(3);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::AuthorizationNotYetValid.into())
        );
    }

    #[test]
    fn transfer_with_authorization_expired() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        // Move further into the future so that valid_before is in the past.
        env.advance_block_time(60_000);

        let valid_after: u64 = 0;
        let valid_before: u64 = 10;
        let amount: U256 = 100u64.into();
        let nonce = fresh_nonce(4);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::AuthorizationExpired.into())
        );
    }

    #[test]
    fn transfer_with_authorization_invalid_signature() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(5);

        // Alice signs for an amount of 100 ...
        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            U256::from(100u64),
            valid_after,
            valid_before,
            &nonce
        );

        // ... but charlie submits with a different amount, so the digest the
        // contract recomputes won't match the signature.
        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                U256::from(200u64),
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::InvalidSignature.into())
        );
    }

    #[test]
    fn transfer_with_authorization_public_key_mismatch() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            ..
        } = setup();

        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let amount: U256 = 100u64.into();
        let nonce = fresh_nonce(6);

        // Sign with bob's key while declaring alice as `from`.
        let bob_pubkey = env.public_key(&bob);
        let signature = sign_transfer_authorization(
            &env,
            &bob,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                bob_pubkey,
                signature
            ),
            Err(Error::InvalidPublicKey.into())
        );
    }

    #[test]
    fn receive_with_authorization_happy_path() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            alice_pubkey,
            ..
        } = setup();

        let amount: U256 = 250u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(7);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            RECEIVE_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        // Bob (the recipient) must be the caller for receive_with_authorization.
        env.set_caller(bob);
        wrapper.receive_with_authorization(
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            nonce.clone(),
            alice_pubkey,
            signature
        );

        assert_eq!(
            wrapper.balance_of(&alice),
            U256::from(INITIAL_SUPPLY) - amount
        );
        assert_eq!(wrapper.balance_of(&bob), amount);
        assert!(wrapper.authorization_state(alice, nonce));
    }

    #[test]
    fn receive_with_authorization_wrong_caller_reverts() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let amount: U256 = 250u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(8);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            RECEIVE_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        // Charlie tries to relay even though `to` is bob.
        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_receive_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::InvalidCaller.into())
        );
    }

    #[test]
    fn cancel_authorization_happy_path() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let nonce = fresh_nonce(9);
        let cancel_signature =
            sign_cancel_authorization(&env, &alice, wrapper.address(), alice, &nonce);

        env.set_caller(charlie);
        wrapper.cancel_authorization(alice, nonce.clone(), alice_pubkey.clone(), cancel_signature);

        assert!(wrapper.authorization_state(alice, nonce.clone()));
        assert!(env.emitted_event(
            &wrapper,
            AuthorizationCanceled {
                authorizer: alice,
                nonce: nonce.clone()
            }
        ));

        // A subsequent transfer reusing the cancelled nonce is rejected.
        let amount: U256 = 1u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let transfer_signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                transfer_signature
            ),
            Err(Error::NonceAlreadyUsed.into())
        );
    }

    #[test]
    fn cancel_authorization_already_used_reverts() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let amount: U256 = 100u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(10);

        // Consume the nonce via a successful transfer first.
        let transfer_signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );
        env.set_caller(charlie);
        wrapper.transfer_with_authorization(
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            nonce.clone(),
            alice_pubkey.clone(),
            transfer_signature
        );

        // Cancelling an already-consumed nonce reverts.
        let cancel_signature =
            sign_cancel_authorization(&env, &alice, wrapper.address(), alice, &nonce);
        assert_eq!(
            wrapper.try_cancel_authorization(alice, nonce, alice_pubkey, cancel_signature),
            Err(Error::AuthorizationUsed.into())
        );
    }

    #[test]
    fn cancel_authorization_with_mismatched_public_key_reverts() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            ..
        } = setup();

        let nonce = fresh_nonce(11);

        // Bob signs a cancel message that names alice as the authorizer,
        // using his own keypair.
        let bob_pubkey = env.public_key(&bob);
        let cancel_signature =
            sign_cancel_authorization(&env, &bob, wrapper.address(), alice, &nonce);

        // Submitting with authorizer=alice but public_key=bob_pubkey must fail,
        // otherwise an attacker could burn alice's nonces.
        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_cancel_authorization(alice, nonce.clone(), bob_pubkey, cancel_signature),
            Err(Error::InvalidPublicKey.into())
        );

        // The nonce must remain unused so alice's pending authorization is not bricked.
        assert!(!wrapper.authorization_state(alice, nonce));
    }

    #[test]
    fn transfer_with_authorization_rejects_appended_byte_nonce_alias() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let amount: U256 = 100u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        let nonce = fresh_nonce(20);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        // First redemption with the canonical 32-byte nonce succeeds.
        env.set_caller(charlie);
        wrapper.transfer_with_authorization(
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            nonce.clone(),
            alice_pubkey.clone(),
            signature.clone()
        );

        // The same signature with a byte appended to the nonce hashes to the
        // same 32-byte digest (the digest only covers 32 bytes) but produces a
        // distinct replay-protection storage key. It must be rejected outright
        // rather than allowing a second redemption.
        let mut aliased = nonce.to_vec();
        aliased.push(0);
        let aliased_nonce = Bytes::from(aliased);

        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                aliased_nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::InvalidNonceLength.into())
        );

        // The funds moved exactly once.
        assert_eq!(wrapper.balance_of(&bob), amount);
    }

    #[test]
    fn transfer_with_authorization_rejects_short_nonce() {
        let Setup {
            env,
            mut wrapper,
            alice,
            bob,
            charlie,
            alice_pubkey
        } = setup();

        let amount: U256 = 100u64.into();
        let valid_after: u64 = 0;
        let valid_before: u64 = u64::MAX;
        // A 31-byte nonce that the digest would right-pad to 32 bytes.
        let nonce = Bytes::from(vec![21u8; 31]);

        let signature = sign_transfer_authorization(
            &env,
            &alice,
            wrapper.address(),
            TRANSFER_WITH_AUTHORIZATION_TYPEHASH,
            alice,
            bob,
            amount,
            valid_after,
            valid_before,
            &nonce
        );

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_transfer_with_authorization(
                alice,
                bob,
                amount,
                valid_after,
                valid_before,
                nonce,
                alice_pubkey,
                signature
            ),
            Err(Error::InvalidNonceLength.into())
        );
    }

    #[test]
    fn cancel_authorization_rejects_nonstandard_nonce() {
        let Setup {
            env,
            mut wrapper,
            alice,
            charlie,
            alice_pubkey,
            ..
        } = setup();

        // An oversized (33-byte) nonce aliases a 32-byte cancel digest.
        let nonce = Bytes::from(vec![22u8; 33]);
        let cancel_signature =
            sign_cancel_authorization(&env, &alice, wrapper.address(), alice, &nonce);

        env.set_caller(charlie);
        assert_eq!(
            wrapper.try_cancel_authorization(alice, nonce.clone(), alice_pubkey, cancel_signature),
            Err(Error::InvalidNonceLength.into())
        );

        // The aliased nonce must not be recorded as used.
        assert!(!wrapper.authorization_state(alice, nonce));
    }

    fn sign_transfer_authorization(
        env: &HostEnv,
        signer: &Address,
        contract_address: Address,
        typehash: [u8; 32],
        from: Address,
        to: Address,
        amount: U256,
        valid_after: u64,
        valid_before: u64,
        nonce: &Bytes
    ) -> Bytes {
        let mut value_bytes = [0u8; 32];
        amount.to_big_endian(&mut value_bytes);

        let mut nonce_padded = [0u8; 32];
        let len = nonce.len().min(32);
        nonce_padded[..len].copy_from_slice(&nonce[..len]);

        let mut encoded_data = Vec::with_capacity(6 * 32);
        encoded_data.extend(crate::eip712::encode_address(from));
        encoded_data.extend(crate::eip712::encode_address(to));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_after));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_before));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce_padded));

        let domain = crate::eip712::domain_separator(
            TOKEN_NAME,
            DOMAIN_VERSION,
            CHAIN_NAME.to_string(),
            contract_address
        );
        let message_hash = crate::eip712::hash_typed_data(domain, typehash, encoded_data);
        let message = Bytes::from(message_hash.to_vec());

        env.sign_message(&message, signer)
    }

    fn sign_cancel_authorization(
        env: &HostEnv,
        signer: &Address,
        contract_address: Address,
        authorizer: Address,
        nonce: &Bytes
    ) -> Bytes {
        let mut nonce_padded = [0u8; 32];
        let len = nonce.len().min(32);
        nonce_padded[..len].copy_from_slice(&nonce[..len]);

        let mut encoded_data = Vec::with_capacity(64);
        encoded_data.extend(crate::eip712::encode_address(authorizer));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce_padded));

        let domain = crate::eip712::domain_separator(
            TOKEN_NAME,
            DOMAIN_VERSION,
            CHAIN_NAME.to_string(),
            contract_address
        );
        let message_hash =
            crate::eip712::hash_typed_data(domain, CANCEL_AUTHORIZATION_TYPEHASH, encoded_data);
        let message = Bytes::from(message_hash.to_vec());

        env.sign_message(&message, signer)
    }
}
