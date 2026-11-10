use super::*;
use syn::parse_quote;

#[test]
fn flags_are_read_across_attributes() {
    let attrs = OdraAttrs::parse(&[
        parse_quote!(#[odra(payable)]),
        parse_quote!(#[yoyo(abc)]),
        parse_quote!(#[odra(offchain)])
    ])
    .unwrap();
    assert!(attrs.is_payable());
    assert!(!attrs.is_non_reentrant());
    assert!(attrs.is_offchain());
    assert_eq!(
        attrs
            .each_attribute()
            .map(|(_, name)| name)
            .collect::<Vec<_>>(),
        vec!["payable", "offchain"]
    );
}

#[test]
fn no_odra_attribute_means_no_flags() {
    let attrs = OdraAttrs::parse(&[parse_quote!(#[yoyo(abc)])]).unwrap();
    assert!(!attrs.is_payable());
    assert!(!attrs.is_non_reentrant());
    assert!(!attrs.is_offchain());
    assert!(attrs.offchain_attribute().is_none());
}

#[test]
fn other_attributes_are_kept() {
    let other: syn::Attribute = parse_quote!(#[yoyo(abc)]);
    let kept = other_attributes([parse_quote!(#[odra(payable)]), other.clone()]);
    assert_eq!(kept, vec![other]);
}

// Wrong attributes (unknown, duplicate, malformed) are covered by the compile-fail tests of
// the `odra` crate (`tests/macros/fail`), which check the message and its span.
