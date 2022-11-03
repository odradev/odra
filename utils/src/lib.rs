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
pub fn camel_to_snake(text: &str) -> String {
    text.from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

/// Calculates the absolute position of the event. Accepts both positive and negative indexing.
///
/// # Examples
///
/// ```
/// use odra_utils::event_absolute_position;
/// use odra_types::event::EventError;
///
/// assert_eq!(event_absolute_position(10, 0), Some(0));
/// assert_eq!(event_absolute_position(10, -1), Some(9));
/// assert_eq!(event_absolute_position(10, 10), None);
/// ```
pub fn event_absolute_position(len: usize, index: i32) -> Option<usize> {
    if index.is_negative() {
        let abs_idx = index.wrapping_abs() as usize;
        if abs_idx > len {
            return None;
        }
        Some(len - abs_idx)
    } else {
        if index as usize >= len {
            return None;
        }
        Some(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::{camel_to_snake, event_absolute_position};

    #[test]
    fn camel_to_snake_works() {
        assert_eq!("owned_token", camel_to_snake("OwnedToken"));
        assert_eq!("ownable", camel_to_snake("Ownable"));
        assert_eq!("erc20", camel_to_snake("Erc20"));
        assert_eq!(
            "erc20_implementation",
            camel_to_snake("Erc20Implementation")
        );
    }

    #[test]
    fn event_absolute_position_works() {
        assert_eq!(event_absolute_position(0, 1), None);
        assert_eq!(event_absolute_position(10, 10), None);
        assert_eq!(event_absolute_position(10, -11), None);
        assert_eq!(event_absolute_position(10, 0), Some(0));
        assert_eq!(event_absolute_position(10, 1), Some(1));
        assert_eq!(event_absolute_position(10, -1), Some(9));
        assert_eq!(event_absolute_position(10, -2), Some(8));
        assert_eq!(event_absolute_position(10, -10), Some(0));
    }
}
