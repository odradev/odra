use convert_case::{Boundary, Case, Casing};
use quote::ToTokens;

pub fn camel_to_snake<T: ToString>(text: T) -> String {
    text.to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

pub fn eq<A: ToTokens, B: ToTokens>(a: A, b: B) -> bool {
    a.to_token_stream().to_string() == b.to_token_stream().to_string()
}
