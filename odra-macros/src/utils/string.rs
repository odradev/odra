use convert_case::{Boundary, Case, Casing};

pub fn camel_to_snake<T: ToString>(text: T) -> String {
    text.to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}
