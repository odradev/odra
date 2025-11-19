//! Account management methods.

use crate::casper_client::Result;
use crate::error::LivenetError;
use casper_client::JsonRpcId;
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::{PublicKey, SecretKey};
use itertools::Itertools;
use odra_core::casper_types::crypto::sign;
use odra_core::prelude::*;
use rand::random;

/// Account management methods implementation for CasperClient.
impl super::CasperClient {
    /// Sets the amount of gas for the next deployment.
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas.into();
    }

    /// Public key of the client account.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.configuration.secret_keys[self.active_account])
    }

    /// Public key of the account address.
    pub fn address_public_key(&self, address: &Address) -> PublicKey {
        PublicKey::from(self.address_secret_key(address))
    }

    /// Secret key of the client account.
    pub fn secret_key(&self) -> &SecretKey {
        &self.configuration.secret_keys[self.active_account]
    }

    /// Signs the message using keys associated with an address.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Result<Bytes> {
        let secret_key = self.address_secret_key(address);
        let public_key = &PublicKey::from(secret_key);
        let signature = sign(message, secret_key, public_key)
            .to_bytes()
            .map_err(|_| LivenetError::SerializationError)?;
        Ok(Bytes::from(signature))
    }

    /// Address of the client account.
    pub fn caller(&self) -> Address {
        Address::from(self.public_key())
    }

    /// Address of the account loaded to the client.
    pub fn get_account(&self, index: usize) -> Address {
        if index >= self.secret_keys().len() {
            panic!("Key for account with index {} is not loaded", index);
        }
        Address::from(PublicKey::from(&self.secret_keys()[index]))
    }

    /// Sets the caller account.
    pub fn set_caller(&mut self, address: Address) {
        match self
            .secret_keys()
            .iter()
            .find_position(|key| Address::from(PublicKey::from(*key)) == address)
        {
            Some((index, _)) => {
                self.active_account = index;
            }
            None => panic!("Key for address {:?} is not loaded", address)
        }
    }

    fn address_secret_key(&self, address: &Address) -> &SecretKey {
        match self
            .secret_keys()
            .iter()
            .find(|key| Address::from(PublicKey::from(*key)) == *address)
        {
            Some(secret_key) => secret_key,
            None => panic!("Key for address {:?} is not loaded", address)
        }
    }

    fn secret_keys(&self) -> &Vec<SecretKey> {
        &self.configuration.secret_keys
    }

    pub(crate) fn rpc_id(&self) -> String {
        let random_number: u32 = random();
        random_number.to_string()
    }

    pub(crate) fn rpc_id_typed(&self) -> JsonRpcId {
        JsonRpcId::String("1".to_string())
    }
}
