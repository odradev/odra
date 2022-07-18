use convert_case::{Case, Casing, Boundary};

pub fn camel_to_snake(text: &str) -> String {
    text
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake)
}

#[cfg(test)]
mod tests {
    use crate::camel_to_snake;

    #[test]
    fn camel_to_snake_works() {
        assert_eq!("owned_token", camel_to_snake("OwnedToken"));
        assert_eq!("ownable", camel_to_snake("Ownable"));
        assert_eq!("erc20", camel_to_snake("Erc20"));
        assert_eq!("erc20_implementation", camel_to_snake("Erc20Implementation"));
    }
}
