#![allow(clippy::too_many_arguments)]
//! ERC-3009 implementation for Casper, allowing gasless token transfers via off-chain signatures.

use crate::{cep18_token::Cep18, eip712};
use casper_eip_712::DomainSeparator;
use odra::{
    casper_types::{bytesrepr::Bytes, PublicKey, U256},
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

/// Emitted when an authorization is used via `transfer_with_authorization` or `receive_with_authorization`.
#[odra::event]
pub struct AuthorizationUsed {
    authorizer: Address,
    nonce: Bytes
}

/// Emitted when an authorization is canceled via `cancel_authorization`.
#[odra::event]
pub struct AuthorizationCanceled {
    authorizer: Address,
    nonce: Bytes
}

/// Errors for ERC-3009 operations.
#[odra::odra_error]
pub enum Error {
    /// The provided nonce has already been used or canceled.
    NonceAlreadyUsed = 1,
    /// The current block time is past the `valid_before` timestamp.
    AuthorizationExpired = 2,
    /// The current block time is before the `valid_after` timestamp.
    AuthorizationNotYetValid = 3,
    /// The provided signature is invalid.
    InvalidSignature = 4,
    /// The provided public key does not match the `from` address.
    InvalidFromAddress = 5,
    /// The provided public key is invalid.
    InvalidPublicKey = 6,
    /// The caller of `receive_with_authorization` is not the `to` address.
    InvalidCaller = 7,
    /// The authorization has already been used (for cancellation).
    AuthorizationUsed = 8
}

/// ERC-3009 implementation for Casper, allowing gasless token transfers via off-chain signatures.
#[odra::module(events = [AuthorizationUsed, AuthorizationCanceled], errors = Error)]
pub struct ERC3009 {
    used_nonces: Mapping<(Address, Bytes), bool>,
    chain_name: Var<String>,
    token: SubModule<Cep18>
}

#[odra::module]
impl ERC3009 {
    /// Initializes the module with the given chain name (used in EIP-712 domain) and the address of the CEP-18 token contract.
    pub fn init(&mut self, chain_name: String) {
        self.chain_name.set(chain_name);
    }

    /// Check the authorization state for a given authorizer and nonce.
    pub fn authorization_state(&self, authorizer: Address, nonce: Bytes) -> bool {
        self.used_nonces.get_or_default(&(authorizer, nonce))
    }

    /// Authorizes a transfer from `from` to `to` if the signature is valid and the authorization is not expired or used.
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

    /// Allows a spender to transfer tokens on behalf of the token holder, given an authorization signed by the token holder.
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

    /// Cancels an authorization if it has not been used yet.
    pub fn cancel_authorization(
        &mut self,
        authorizer: Address,
        nonce: Bytes,
        public_key: PublicKey,
        signature: Bytes
    ) {
        if self.authorization_state(authorizer, nonce.clone()) {
            self.env().revert(Error::AuthorizationUsed);
        }

        let message = self.build_cancel_message(authorizer, &nonce);
        if !self
            .env()
            .verify_signature(&message, &signature, &public_key)
        {
            self.env().revert(Error::InvalidSignature);
        }

        self.used_nonces.set(&(authorizer, nonce.clone()), true);
        self.env()
            .emit_event(AuthorizationCanceled { authorizer, nonce });
    }
}

impl ERC3009 {
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
        // 1. Replay protection
        if self.used_nonces.get_or_default(&(from, nonce.clone())) {
            self.env().revert(Error::NonceAlreadyUsed);
        }

        // 2. block_time
        let now_secs = self.env().get_block_time_secs();

        // 3. Check valid_after
        if now_secs <= valid_after {
            self.env().revert(Error::AuthorizationNotYetValid);
        }

        // 4. Check valid_before
        if now_secs >= valid_before {
            self.env().revert(Error::AuthorizationExpired);
        }

        // 5. Verify that public_key matches the `from` address
        let derived_address = Address::from(public_key.clone());
        if derived_address != from {
            self.env().revert(Error::InvalidPublicKey);
        }

        // 6. Build message and verify signature
        let message = self.build_authorization_message(
            typehash,
            from,
            to,
            &amount,
            valid_after,
            valid_before,
            &nonce
        );

        if !self
            .env()
            .verify_signature(&message, &signature, &public_key)
        {
            self.env().revert(Error::InvalidSignature);
        }

        // 7. Mark nonce as used
        self.used_nonces.set(&(from, nonce.clone()), true);

        // 8. Emit event
        self.env().emit_event(AuthorizationUsed {
            authorizer: from,
            nonce
        });

        // 9. Execute transfer (raw_transfer takes refs)
        self.token.raw_transfer(&from, &to, &amount);
    }

    /// Build the EIP-712 hash for a transfer authorization.
    fn build_authorization_message(
        &self,
        typehash: [u8; 32],
        from: Address,
        to: Address,
        amount: &U256,
        valid_after: u64,
        valid_before: u64,
        nonce: &[u8]
    ) -> Bytes {
        let mut value_bytes = [0u8; 32];
        amount.to_big_endian(&mut value_bytes);

        let mut nonce_padded = [0u8; 32];
        let len = nonce.len().min(32);
        nonce_padded[..len].copy_from_slice(&nonce[..len]);

        let mut encoded_data = Vec::with_capacity(6 * 32);
        encoded_data.extend(eip712::encode_address(from));
        encoded_data.extend(eip712::encode_address(to));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_after));
        encoded_data.extend(casper_eip_712::encode_uint64(valid_before));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce_padded));
        let domain = self.domain_separator();

        Bytes::from(crate::eip712::hash_typed_data(domain, typehash, encoded_data).to_vec())
    }

    fn build_cancel_message(&self, authorizer: Address, nonce: &[u8]) -> Bytes {
        let mut nonce_padded = [0u8; 32];
        let len = nonce.len().min(32);
        nonce_padded[..len].copy_from_slice(&nonce[..len]);

        let mut encoded_data = Vec::with_capacity(64);
        encoded_data.extend(eip712::encode_address(authorizer));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce_padded));
        let domain = self.domain_separator();

        Bytes::from(
            eip712::hash_typed_data(domain, CANCEL_AUTHORIZATION_TYPEHASH, encoded_data).to_vec()
        )
    }

    fn domain_separator(&self) -> DomainSeparator {
        let self_address = self.env().self_address();
        let name = self.token.name();
        let chain_id = self.chain_name.get().unwrap_or_revert(self);
        eip712::domain_separator(&name, chain_id, self_address)
    }
}

/// Wrapper contract that combines ERC-3009 functionality with a CEP-18 token for testing purposes.
#[odra::module]
pub struct ERC3009Wrapper {
    erc3009: SubModule<ERC3009>,
    token: SubModule<Cep18>
}

/// Wrapper contract that combines ERC-3009 functionality with a CEP-18 token for testing purposes.
/// In a real deployment, the ERC-3009 module would likely be separate and interact with an existing token contract.
#[odra::module]
impl ERC3009Wrapper {
    /// Initializes the wrapper by deploying the ERC-3009 module and the CEP-18 token, and setting up the EIP-712 domain.
    pub fn init(
        &mut self,
        chain_name: String,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256
    ) {
        self.erc3009.init(chain_name);
        self.token.init(symbol, name, decimals, initial_supply);
    }

    delegate! {
        to self.erc3009 {
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
    const CHAIN_NAME: &str = "casper-test";

    struct Setup {
        env: HostEnv,
        wrapper: ERC3009WrapperHostRef,
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

        let wrapper = ERC3009Wrapper::deploy(
            &env,
            ERC3009WrapperInitArgs {
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

        let domain =
            crate::eip712::domain_separator(TOKEN_NAME, CHAIN_NAME.to_string(), contract_address);
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

        let domain =
            crate::eip712::domain_separator(TOKEN_NAME, CHAIN_NAME.to_string(), contract_address);
        let message_hash =
            crate::eip712::hash_typed_data(domain, CANCEL_AUTHORIZATION_TYPEHASH, encoded_data);
        let message = Bytes::from(message_hash.to_vec());

        env.sign_message(&message, signer)
    }
}
