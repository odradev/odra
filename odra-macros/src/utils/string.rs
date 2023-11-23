use convert_case::{Boundary, Case, Casing};

/// Converts a camel-cased &str to String.
///
/// # Example
///
/// ```
/// use odra_utils::camel_to_snake;
///
/// let camel = "ContractName";
/// let result = camel_to_snake(camel);
///
/// assert_eq!(&result, "contract_name");
/// ```
pub fn camel_to_snake<T: ToString>(text: T) -> String {
    text.to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}
