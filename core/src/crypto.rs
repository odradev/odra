//! Cryptographic utilities.

use crate::prelude::*;
use casper_types::account::AccountHash;
use casper_types::{PublicKey, SecretKey};

/// Generates a key pair map of the given size.
/// The key pairs are generated deterministically from the index.
///
/// # Arguments
///
/// * `size` - The number of key pairs to generate.
///
/// # Returns
///
/// A map containing the generated key pairs.
pub fn generate_key_pairs(amount: u8) -> BTreeMap<Address, (SecretKey, PublicKey)> {
    let mut accounts = BTreeMap::new();
    for i in 0..amount {
        // Create keypair.
        let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap_or_else(|_| {
            panic!(
                "Couldn't construct a secret key from {}. This shouldn't happen!",
                i
            )
        });
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);

        let address = account_addr.try_into().unwrap_or_else(|_| {
            panic!("Couldn't convert AccountHash to Address. This shouldn't happen!")
        });

        // Create a GenesisAccount.
        accounts.insert(address, (secret_key, public_key));
    }
    accounts
}
