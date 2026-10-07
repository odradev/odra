//! The value conversions of [odra_schema::codec], with errors as the CLI reports them.

use odra::schema::casper_contract_schema::Type;
pub(crate) use odra_schema::codec::{
    format_type_hint, into_bytes, named_cl_type_to_cl_type, to_bytes_or_err, Error
};

use crate::{cmd::args::ArgsError, custom_types::CustomTypeSet};

#[cfg(test)]
mod decoder_tests;

pub(crate) fn decode<'a>(
    bytes: &'a [u8],
    ty: &Type,
    types: &'a CustomTypeSet
) -> Result<(String, &'a [u8]), ArgsError> {
    Ok(odra_schema::codec::decode(bytes, ty, types)?)
}

pub(crate) fn decode_event(bytes: &[u8], types: &CustomTypeSet) -> Result<String, ArgsError> {
    Ok(odra_schema::codec::decode_event(bytes, types)?)
}

pub(crate) fn format_variant_list(variants: &[(String, u16)]) -> String {
    variants
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join("|")
}
