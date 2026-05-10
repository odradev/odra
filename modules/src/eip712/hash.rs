use casper_eip_712::DomainSeparator;
use odra::prelude::*;

pub(crate) fn hash_typed_data(
    domain: DomainSeparator,
    typehash: [u8; 32],
    encoded_data: Vec<u8>
) -> [u8; 32] {
    let mut data = [0u8; 66];
    data[0] = 0x19;
    data[1] = 0x01;
    data[2..34].copy_from_slice(&domain.separator_hash());
    let struct_hash = {
        let mut struct_data = Vec::with_capacity(32 + encoded_data.len());
        struct_data.extend_from_slice(&typehash);
        struct_data.extend_from_slice(&encoded_data);
        casper_eip_712::keccak256(&struct_data)
    };

    data[34..66].copy_from_slice(&struct_hash);
    casper_eip_712::keccak256(&data)
}
