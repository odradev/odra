use std::char;

use casper_eip_712::Address as Eip712Address;
use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::{KeyTag, PublicKey};
use odra::host::Deployer;
use odra::{casper_types::U256, host::HostEnv, prelude::*};
use odra_examples::contracts::gasless_cep18::{GaslessCep18, GaslessCep18InitArgs};

fn main() {
    let env: HostEnv = odra_casper_livenet_env::env();
    env.set_gas(500_000_000_000u64);

    let chain_name = std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME")
        .unwrap_or_else(|_| "casper-test".to_string());

    let mut contract = GaslessCep18::deploy(
        &env,
        GaslessCep18InitArgs {
            chain_name: chain_name.clone()
        }
    );
    let contract_address = contract.address();

    let alice = env.get_account(0);
    let bob = env.get_account(1);
    let charlie = env.get_account(2);
    let signed_message = sign_transfer_auth(&env, &chain_name, &contract_address, &alice, &bob);

    let sig_hex: String = signed_message
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    println!("Signature: 0x{}", sig_hex);

    env.set_caller(charlie);

    let amount = U256::from(1000);
    let valid_after = 0;
    let valid_before = u64::MAX;
    let nonce = Bytes::from([0u8; 32].to_vec());
    let signature = signed_message;
    let pk = env.public_key(&alice);

    let result = contract.try_transfer_with_authorization(
        alice,
        bob,
        amount,
        valid_after,
        valid_before,
        nonce,
        pk,
        signature
    );

    match result {
        Ok(_) => println!("Transfer with authorization succeeded"),
        Err(e) => println!("Transfer with authorization failed: {:?}", e)
    }
}

fn sign_transfer_auth(
    env: &HostEnv,
    chain_name: &str,
    contract_address: &Address,
    signer: &Address,
    recipient: &Address
) -> Bytes {
    let auth = TransferWithAuthorization {
        from: *signer,
        to: *recipient,
        value: U256::from(1000),
        valid_after: 0,
        valid_before: u64::MAX,
        nonce: [0u8; 32]
    };
    let domain = casper_eip_712::DomainBuilder::new()
        .name("USDC")
        .custom_field(
            "chain_name",
            casper_eip_712::DomainFieldValue::String(chain_name.to_string())
        )
        .custom_field(
            "contract_package_hash",
            casper_eip_712::DomainFieldValue::Bytes32(contract_address.value())
        )
        .build();

    let message = Bytes::from(casper_eip_712::hash_typed_data(&domain, &auth).to_vec());
    env.set_caller(*signer);
    env.sign_message(&message, signer)
}

// keccak256("TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)")

struct TransferWithAuthorization {
    from: Address,
    to: Address,
    value: U256,
    valid_after: u64,
    valid_before: u64,
    nonce: [u8; 32]
}

impl casper_eip_712::Eip712Struct for TransferWithAuthorization {
    fn type_string() -> &'static str {
        "TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)"
    }

    fn encode_data(&self) -> Vec<u8> {
        let mut value_bytes = [0u8; 32];
        self.value.to_big_endian(&mut value_bytes);

        let mut nonce_padded = [0u8; 32];
        let len = self.nonce.len().min(32);
        nonce_padded[..len].copy_from_slice(&self.nonce[..len]);

        let mut encoded_data = Vec::with_capacity(6 * 32);
        encoded_data.extend(casper_eip_712::encode_address(
            odra_address_to_eip712_address(self.from)
        ));
        encoded_data.extend(casper_eip_712::encode_address(
            odra_address_to_eip712_address(self.to)
        ));
        encoded_data.extend(casper_eip_712::encode_uint256(value_bytes));
        encoded_data.extend(casper_eip_712::encode_uint64(self.valid_after));
        encoded_data.extend(casper_eip_712::encode_uint64(self.valid_before));
        encoded_data.extend(casper_eip_712::encode_bytes32(nonce_padded));
        encoded_data
    }
}

fn odra_address_to_eip712_address(addr: Address) -> Eip712Address {
    let mut bytes = [0u8; 33];
    match addr {
        Address::Account(_) => bytes[0] = KeyTag::Account as u8,
        Address::Contract(_) => bytes[0] = KeyTag::Hash as u8
    }
    bytes[1..33].copy_from_slice(&addr.value());
    Eip712Address::Casper(bytes)
}
