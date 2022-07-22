use convert_case::{Boundary, Case, Casing};
use odra_types::event;

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
/// assert_eq!(event_absolute_position(10, 0), Ok(0));
/// assert_eq!(event_absolute_position(10, -1), Ok(9));
/// assert_eq!(event_absolute_position(10, 10), Err(EventError::IndexOutOfBounds));
/// ```
pub fn event_absolute_position(len: usize, index: i32) -> Result<usize, event::EventError> {
    if index.is_negative() {
        let abs_idx = index.wrapping_abs() as usize;
        if abs_idx > len {
            return Err(event::EventError::IndexOutOfBounds);
        }
        Ok(len - abs_idx)
    } else {
        if index as usize >= len {
            return Err(event::EventError::IndexOutOfBounds);
        }
        Ok(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::{camel_to_snake, event_absolute_position};
    use odra_types::event;

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
        assert_eq!(
            event_absolute_position(0, 1),
            Err(event::EventError::IndexOutOfBounds)
        );
        assert_eq!(
            event_absolute_position(10, 10),
            Err(event::EventError::IndexOutOfBounds)
        );
        assert_eq!(
            event_absolute_position(10, -11),
            Err(event::EventError::IndexOutOfBounds)
        );
        assert_eq!(event_absolute_position(10, 0), Ok(0));
        assert_eq!(event_absolute_position(10, 1), Ok(1));
        assert_eq!(event_absolute_position(10, -1), Ok(9));
        assert_eq!(event_absolute_position(10, -2), Ok(8));
        assert_eq!(event_absolute_position(10, -10), Ok(0));
    }
}
