use odra::prelude::*;

pub(crate) fn domain_separator(
    name: &str,
    chain_id: String,
    contract_address: Address
) -> casper_eip_712::DomainSeparator {
    casper_eip_712::DomainBuilder::new()
        .name(name)
        .custom_field(
            "chain_id",
            casper_eip_712::DomainFieldValue::String(chain_id)
        )
        .custom_field(
            "contract_package_hash",
            casper_eip_712::DomainFieldValue::Bytes32(contract_address.value())
        )
        .build()
}
