pub trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl ToSnakeCase for str {
    fn to_snake_case(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut words = 0;
        for (index, char) in self.char_indices() {
            if index > 0 && char.is_uppercase() {
                words += 1;
            }
            if words > 0 && char.is_uppercase() {
                result.push('_');
            }
            result.push(char.to_lowercase().to_string().chars().next().unwrap());
        }
        result
    }
}
