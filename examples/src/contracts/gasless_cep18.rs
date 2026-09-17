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

/// Building and signing an EIP-712 `TransferWithAuthorization` for [GaslessCep18] on the host side
/// (tests, scripts, the examples' CLI).
///
/// The domain must match what `CEP3009` derives on chain: token name, version `"1"`, the chain
/// name given at `init` and the contract's package hash.
#[cfg(not(target_arch = "wasm32"))]
pub mod authorization {
    use casper_eip_712::Address as Eip712Address;
    use odra::casper_types::bytesrepr::Bytes;
    use odra::casper_types::{KeyTag, U256};
    use odra::host::HostEnv;
    use odra::prelude::*;

    /// The token name [GaslessCep18::init] stores; part of the EIP-712 domain.
    pub const TOKEN_NAME: &str = "USDC";
    /// The EIP-712 domain version `CEP3009` uses.
    pub const DOMAIN_VERSION: &str = "1";

    /// A `TransferWithAuthorization` message.
    pub struct TransferWithAuthorization {
        pub from: Address,
        pub to: Address,
        pub value: U256,
        pub valid_after: u64,
        pub valid_before: u64,
        pub nonce: [u8; 32]
    }

    /// Signs `auth` with the key of `auth.from`, for the contract deployed at `contract_address`
    /// on the chain named `chain_name`.
    pub fn sign_transfer_authorization(
        env: &HostEnv,
        chain_name: &str,
        contract_address: &Address,
        auth: &TransferWithAuthorization
    ) -> Bytes {
        let domain = casper_eip_712::DomainBuilder::new()
            .name(TOKEN_NAME)
            .version(DOMAIN_VERSION)
            .custom_field(
                "chain_name",
                casper_eip_712::DomainFieldValue::String(chain_name.to_string())
            )
            .custom_field(
                "contract_package_hash",
                casper_eip_712::DomainFieldValue::Bytes32(contract_address.value())
            )
            .build();
        let message = Bytes::from(casper_eip_712::hash_typed_data(&domain, auth).to_vec());
        env.sign_message(&message, &auth.from)
    }

    impl casper_eip_712::Eip712Struct for TransferWithAuthorization {
        fn type_string() -> &'static str {
            "TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)"
        }

        fn encode_data(&self) -> Vec<u8> {
            let mut value_bytes = [0u8; 32];
            self.value.to_big_endian(&mut value_bytes);
            let mut encoded_data = Vec::with_capacity(6 * 32);
            encoded_data.extend(casper_eip_712::encode_address(to_eip712_address(self.from)));
            encoded_data.extend(casper_eip_712::encode_address(to_eip712_address(self.to)));
            encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
            encoded_data.extend(casper_eip_712::encode_uint64(self.valid_after));
            encoded_data.extend(casper_eip_712::encode_uint64(self.valid_before));
            encoded_data.extend(casper_eip_712::encode_bytes32(self.nonce));
            encoded_data
        }
    }

    fn to_eip712_address(addr: Address) -> Eip712Address {
        let mut bytes = [0u8; 33];
        match addr {
            Address::Account(_) => bytes[0] = KeyTag::Account as u8,
            Address::Contract(_) => bytes[0] = KeyTag::Hash as u8
        }
        bytes[1..33].copy_from_slice(&addr.value());
        Eip712Address::Casper(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::authorization::{sign_transfer_authorization, TransferWithAuthorization};
    use super::{GaslessCep18, GaslessCep18InitArgs};
    use core::time::Duration;
    use odra::casper_types::bytesrepr::Bytes;
    use odra::casper_types::U256;
    use odra::host::Deployer;
    use odra::prelude::*;

    /// The same flow the CLI's `gasless` scenario runs on livenet: Alice signs, Charlie submits.
    #[test]
    fn transfer_with_authorization_signed_on_the_host() {
        let env = odra_test::env();
        let chain_name = "casper-net-1";
        let mut contract = GaslessCep18::deploy(
            &env,
            GaslessCep18InitArgs {
                chain_name: chain_name.to_string()
            }
        );
        let (alice, bob, charlie) = (env.get_account(0), env.get_account(1), env.get_account(2));
        let auth = TransferWithAuthorization {
            from: alice,
            to: bob,
            value: U256::from(1000),
            valid_after: 0,
            valid_before: u64::MAX,
            nonce: [0u8; 32]
        };
        let signature = sign_transfer_authorization(&env, chain_name, &contract.address(), &auth);

        // `valid_after` is exclusive and the VMs start at block time 0.
        env.advance_block_time(Duration::from_secs(1));
        env.set_caller(charlie);
        contract.transfer_with_authorization(
            alice,
            bob,
            auth.value,
            auth.valid_after,
            auth.valid_before,
            Bytes::from(auth.nonce.to_vec()),
            env.public_key(&alice),
            signature
        );
        assert_eq!(contract.balance_of(&bob), U256::from(1000));
    }

    #[test]
    fn a_signature_for_another_domain_is_rejected() {
        let env = odra_test::env();
        let mut contract = GaslessCep18::deploy(
            &env,
            GaslessCep18InitArgs {
                chain_name: "casper-net-1".to_string()
            }
        );
        let (alice, bob) = (env.get_account(0), env.get_account(1));
        let auth = TransferWithAuthorization {
            from: alice,
            to: bob,
            value: U256::from(1000),
            valid_after: 0,
            valid_before: u64::MAX,
            nonce: [0u8; 32]
        };
        // Signed for a different chain name than the contract was initialized with.
        let signature =
            sign_transfer_authorization(&env, "casper-test", &contract.address(), &auth);
        let result = contract.try_transfer_with_authorization(
            alice,
            bob,
            auth.value,
            auth.valid_after,
            auth.valid_before,
            Bytes::from(auth.nonce.to_vec()),
            env.public_key(&alice),
            signature
        );
        assert!(result.is_err());
    }
}
