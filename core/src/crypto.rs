use crate::prelude::*;
use crate::Address;
use casper_types::account::AccountHash;
use casper_types::{PublicKey, SecretKey};

pub fn generate_key_pairs(amount: u8) -> BTreeMap<Address, (SecretKey, PublicKey)> {
    let mut accounts = BTreeMap::new();
    for i in 0..amount {
        // Create keypair.
        let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
        let public_key = PublicKey::from(&secret_key);

        // Create an AccountHash from a public key.
        let account_addr = AccountHash::from(&public_key);

        // Create a GenesisAccount.
        accounts.insert(account_addr.try_into().unwrap(), (secret_key, public_key));
    }
    accounts
}
