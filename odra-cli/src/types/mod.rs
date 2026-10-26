//! The value conversions of [odra_schema::codec], with errors as the CLI reports them.

#[cfg(test)]
mod decoder_tests;

pub(crate) fn format_variant_list(variants: &[(String, u16)]) -> String {
    variants
        .iter()
        .map(|(n, _)| n.as_str())
        .collect::<Vec<_>>()
        .join("|")
}
