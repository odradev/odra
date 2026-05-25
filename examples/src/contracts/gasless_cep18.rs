#![allow(clippy::too_many_arguments)]

use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::{PublicKey, U256};
use odra::prelude::*;
use odra_modules::cep18_token::Cep18;
use odra_modules::cep2612::CEP2612;
use odra_modules::cep3009::CEP3009;

#[odra::module]
pub struct GaslessCep18 {
    token: SubModule<Cep18>,
    cep3009: SubModule<CEP3009>,
    cep2612: SubModule<CEP2612>
}

#[odra::module]
impl GaslessCep18 {
    pub fn init(&mut self, chain_name: String) {
        self.cep3009.init(chain_name.clone());
        self.cep2612.init(chain_name);
        self.token.init(
            "USDC".to_string(),
            "USDC".to_string(),
            8,
            U256::from(1_000_000_000_000u64)
        );
    }

    delegate! {
        to self.token {
            fn name(&self) -> String;
            fn symbol(&self) -> String;
            fn decimals(&self) -> u8;
            fn total_supply(&self) -> U256;
            fn balance_of(&self, owner: &Address) -> U256;
            fn transfer(&mut self, to: &Address, amount: &U256);
            fn approve(&mut self, spender: &Address, amount: &U256);
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
            fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
            fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256);
            fn increase_allowance(&mut self, spender: &Address, inc_by: &U256);
        }

        to self.cep3009 {
            fn authorization_state(&self, authorizer: Address, nonce: Bytes) -> bool;
            fn transfer_with_authorization(&mut self, from: Address, to: Address, amount: U256, valid_after: u64, valid_before: u64, nonce: Bytes, public_key: PublicKey, signature: Bytes);
            fn receive_with_authorization(&mut self, from: Address, to: Address, amount: U256, valid_after: u64, valid_before: u64, nonce: Bytes, public_key: PublicKey, signature: Bytes);
        }

        to self.cep2612 {
            fn permit(&mut self, owner: Address, spender: Address, value: U256, deadline: u64, public_key: PublicKey, signature: Bytes);
        }
    }
}
