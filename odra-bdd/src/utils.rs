use odra::casper_types::U256;

pub fn to_motes(amount: &str) -> U256 {
    if amount.matches('.').count() > 1 {
        panic!("Invalid number format: multiple decimal points");
    }
    let amount = amount.replace("_", "");

    let parts: Vec<&str> = amount.split('.').collect();
    let whole = parts[0].parse::<u64>().unwrap();
    let decimal = parts.get(1).unwrap_or(&"0");
    let decimal_str = format!("{:0<9}", decimal); // pad with zeros up to 9 decimal places
    let decimal_value = decimal_str.parse::<u64>().unwrap();

    U256::from(whole) * U256::from(1_000_000_000) + U256::from(decimal_value)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::types::cspr::CSPRAmount;

    use super::*;
    use odra::uints::ToU512;

    #[test]
    fn test_to_motes() {
        // Whole numbers
        assert_eq!(to_motes("1"), U256::from(1000000000));
        assert_eq!(to_motes("42"), U256::from(42000000000 as u64));
        assert_eq!(to_motes("0"), U256::from(0));

        // Decimal numbers
        assert_eq!(to_motes("0.5"), U256::from(500000000));
        assert_eq!(to_motes("1.23"), U256::from(1230000000 as u64));
        assert_eq!(to_motes("0.000000001"), U256::from(1));

        // Numbers with trailing zeros
        assert_eq!(to_motes("1.500"), U256::from(1500000000));
        assert_eq!(to_motes("0.100000000"), U256::from(100000000));

        // Large numbers
        assert_eq!(to_motes("1000.123456789"), U256::from(1000123456789 as u64));
    }

    #[test]
    #[should_panic]
    fn test_to_motes_invalid_input() {
        to_motes("invalid");
    }

    #[test]
    #[should_panic]
    fn test_to_motes_multiple_dots() {
        to_motes("1.2.3");
    }

    #[test]
    fn test_to_motes_matches_cspr_amount() {
        let test_values = [
            "0",
            "1",
            "0.5",
            "1.23",
            "42",
            "0.000000001",
            "1000.123456789",
            "0.100000000"
        ];

        for value in test_values {
            let our_motes = to_motes(value);
            let cspr_amount = CSPRAmount::from_str(value).unwrap();

            assert_eq!(
                our_motes.to_u512(),
                cspr_amount.amount(),
                "Mismatch for input: {}",
                value
            );
        }
    }
}
