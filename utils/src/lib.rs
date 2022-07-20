use convert_case::{Boundary, Case, Casing};
use odra_types::event;

pub fn camel_to_snake(text: &str) -> String {
    text.from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

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
