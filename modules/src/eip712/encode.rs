use casper_eip_712::Address as Eip712Address;
use odra::{casper_types::KeyTag, prelude::Address};

#[inline(always)]
fn odra_address_to_eip712_address(addr: Address) -> Eip712Address {
    let mut bytes = [0u8; 33];
    match addr {
        Address::Account(_) => bytes[0] = KeyTag::Account as u8,
        Address::Contract(_) => bytes[0] = KeyTag::Hash as u8
    }
    bytes[1..33].copy_from_slice(&addr.value());
    Eip712Address::Casper(bytes)
}

pub(crate) fn encode_address(addr: Address) -> [u8; 32] {
    casper_eip_712::encode_address(odra_address_to_eip712_address(addr))
}
