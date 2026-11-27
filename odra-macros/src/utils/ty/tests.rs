use super::unreferenced_ty;
use quote::ToTokens;
use syn::parse_quote;

fn unreferenced(ty: syn::Type) -> String {
    unreferenced_ty(&ty).to_token_stream().to_string()
}

fn tokens(ty: syn::Type) -> String {
    ty.to_token_stream().to_string()
}

#[test]
fn references_are_dropped_however_deep() {
    assert_eq!(
        unreferenced(parse_quote!(&U256)),
        tokens(parse_quote!(U256))
    );
    assert_eq!(
        unreferenced(parse_quote!(&&U256)),
        tokens(parse_quote!(U256))
    );
    assert_eq!(
        unreferenced(parse_quote!(&&[Address])),
        tokens(parse_quote!(odra::prelude::vec::Vec<Address>))
    );
}

#[test]
fn a_slice_becomes_a_vec_and_other_types_stay() {
    assert_eq!(
        unreferenced(parse_quote!([u8])),
        tokens(parse_quote!(odra::prelude::vec::Vec<u8>))
    );
    assert_eq!(
        unreferenced(parse_quote!(Option<U256>)),
        tokens(parse_quote!(Option<U256>))
    );
}
