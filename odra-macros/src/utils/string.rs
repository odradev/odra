use convert_case::{Boundary, Case, Casing};

pub fn to_lower_case<T: ToString>(t: T) -> String {
    t.to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}
