use odra::prelude::*;

pub(crate) fn domain_separator(
    name: &str,
    version: &str,
    chain_name: String,
    contract_address: Address
) -> casper_eip_712::DomainSeparator {
    casper_eip_712::DomainBuilder::new()
        .name(name)
        .version(version)
        .custom_field(
            "chain_name",
            casper_eip_712::DomainFieldValue::String(chain_name)
        )
        .custom_field(
            "contract_package_hash",
            casper_eip_712::DomainFieldValue::Bytes32(contract_address.value())
        )
        .build()
}
