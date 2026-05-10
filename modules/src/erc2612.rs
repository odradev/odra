use casper_eip_712::DomainSeparator;
use odra::{
    casper_types::{bytesrepr::Bytes, PublicKey, U256},
    prelude::*
};

use crate::{cep18_token::Cep18, eip712};

// keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")
const PERMIT_TYPEHASH: [u8; 32] = [
    0x6e, 0x71, 0xed, 0xae, 0x12, 0xb1, 0xb9, 0x7f, 0x4d, 0x1f, 0x60, 0x37, 0x0f, 0xef, 0x10, 0x10,
    0x5f, 0xa2, 0xfa, 0xae, 0x01, 0x26, 0x11, 0x4a, 0x16, 0x9c, 0x64, 0x84, 0x5d, 0x61, 0x26, 0xc9
];

/// Errors for ERC-2612 permit operations.
#[odra::odra_error]
pub enum Error {
    /// The provided signature is invalid.
    InvalidSignature = 1,
    /// The provided public key does not match the `owner` address.
    InvalidOwnerAddress = 2,
    /// The provided public key is invalid.
    InvalidPublicKey = 3,
    /// The current block time is past the `deadline` timestamp.
    PermitExpired = 4
}

/// A module implementing EIP-2612 permit functionality for a CEP-18 token.
#[odra::module]
pub struct EIP2612 {
    permit_nonces: Mapping<Address, U256>,
    chain_name: Var<String>,
    token: SubModule<Cep18>
}

#[odra::module]
impl EIP2612 {
    /// Initializes the module with the given chain name (e.g., "Casper Mainnet").
    pub fn init(&mut self, chain_name: String) {
        self.chain_name.set(chain_name);
    }

    /// Returns the current nonce for a given owner address, which should be included in the permit signature.
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

        let nonce = self.permit_nonces.get_or_default(&owner);
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

    fn domain_separator(&self) -> DomainSeparator {
        let self_address = self.env().self_address();
        let name = self.token.name();
        let chain_id = self.chain_name.get().unwrap_or_revert(self);
        crate::eip712::domain_separator(&name, chain_id, self_address)
    }

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
