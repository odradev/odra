use cucumber::Parameter;
use odra::casper_types::U256;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct TokenAmount<const DECIMALS: usize> {
    amount: U256,
    precision: usize,
    _phantom: PhantomData<[(); DECIMALS]>
}

impl<const DECIMALS: usize> PartialEq for TokenAmount<DECIMALS> {
    fn eq(&self, other: &Self) -> bool {
        // If amounts are exactly equal, they are always equal
        if self.amount == other.amount {
            return true;
        }

        // Use the minimum precision between the two amounts
        let min_precision = self.precision.min(other.precision);

        if min_precision == DECIMALS {
            return self.amount == other.amount;
        }

        // Calculate the maximum difference allowed based on precision
        let max_diff = U256::from(10u64).pow(U256::from(DECIMALS - min_precision));

        let diff = if self.amount > other.amount {
            self.amount - other.amount
        } else {
            other.amount - self.amount
        };

        diff < max_diff
    }
}

impl<const DECIMALS: usize> PartialOrd for TokenAmount<DECIMALS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

impl<const DECIMALS: usize> TokenAmount<DECIMALS> {
    pub fn new(amount: U256, precision: usize) -> Self {
        TokenAmount {
            amount,
            precision,
            _phantom: PhantomData
        }
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }

    pub fn precision(&self) -> usize {
        self.precision
    }

    fn multiplier() -> U256 {
        U256::from(10u64).pow(U256::from(DECIMALS))
    }
}

impl<const DECIMALS: usize> Deref for TokenAmount<DECIMALS> {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.amount
    }
}

impl<const DECIMALS: usize> Display for TokenAmount<DECIMALS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl<const DECIMALS: usize> FromStr for TokenAmount<DECIMALS> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check if string starts with "roughly" and extract the numeric part
        let (is_rough, s) = if let Some(stripped) = s.strip_prefix("roughly ") {
            (true, stripped.trim())
        } else {
            (false, s)
        };

        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                // No decimal point
                let amount = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                Ok(TokenAmount {
                    amount: amount * Self::multiplier(),
                    // If "roughly" is present, use lower precision, otherwise use full decimal precision
                    precision: if is_rough { 0 } else { DECIMALS },
                    _phantom: PhantomData
                })
            }
            2 => {
                // Has decimal point
                let whole = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                let mut decimal = parts[1].to_string();

                // Set precision to the number of decimal digits provided if "roughly" is present
                // otherwise use full decimal precision
                let precision = if is_rough {
                    decimal.len().min(DECIMALS)
                } else {
                    DECIMALS
                };

                // Pad with zeros if less than DECIMALS decimal places
                while decimal.len() < DECIMALS {
                    decimal.push('0');
                }
                // Truncate if more than DECIMALS decimal places
                decimal.truncate(DECIMALS);
                let fractional = U256::from_dec_str(&decimal).map_err(|e| e.to_string())?;

                Ok(TokenAmount {
                    amount: (whole * Self::multiplier()) + fractional,
                    precision,
                    _phantom: PhantomData
                })
            }
            _ => Err("Invalid token amount format".to_string())
        }
    }
}

impl<const DECIMALS: usize> From<U256> for TokenAmount<DECIMALS> {
    fn from(amount: U256) -> Self {
        TokenAmount {
            amount,
            precision: DECIMALS,
            _phantom: PhantomData
        }
    }
}

impl<const DECIMALS: usize> From<&U256> for TokenAmount<DECIMALS> {
    fn from(amount: &U256) -> Self {
        TokenAmount {
            amount: *amount,
            precision: DECIMALS,
            _phantom: PhantomData
        }
    }
}

#[derive(Debug, Clone, Copy, Parameter, PartialEq, PartialOrd)]
#[param(regex = r"(?:roughly\s+)?\d+(\.\d+)?", name = "usdc")]
pub struct USDCAmount(TokenAmount<6>);

impl Deref for USDCAmount {
    type Target = TokenAmount<6>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for USDCAmount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TokenAmount::<6>::from_str(s).map(USDCAmount)
    }
}

impl From<U256> for USDCAmount {
    fn from(amount: U256) -> Self {
        USDCAmount(TokenAmount::<6>::from(amount))
    }
}

impl From<&U256> for USDCAmount {
    fn from(amount: &U256) -> Self {
        USDCAmount(TokenAmount::<6>::from(amount))
    }
}

#[derive(Debug, Clone, Copy, Parameter, PartialEq, PartialOrd)]
#[param(regex = r"(?:roughly\s+)?\d+(\.\d+)?", name = "wcspr")]
pub struct WrappedCSPRAmount(TokenAmount<9>);

impl Deref for WrappedCSPRAmount {
    type Target = TokenAmount<9>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for WrappedCSPRAmount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TokenAmount::<9>::from_str(s).map(WrappedCSPRAmount)
    }
}

impl From<U256> for WrappedCSPRAmount {
    fn from(amount: U256) -> Self {
        WrappedCSPRAmount(TokenAmount::<9>::from(amount))
    }
}

impl From<&U256> for WrappedCSPRAmount {
    fn from(amount: &U256) -> Self {
        WrappedCSPRAmount(TokenAmount::<9>::from(amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_parsing() {
        let amount1 = USDCAmount::from_str("123.456").unwrap();
        assert_eq!(amount1.0.precision, 6);
        let amount2 = USDCAmount::from_str("123").unwrap();
        assert_eq!(amount2.0.precision, 6);

        let amount3 = USDCAmount::from_str("123.4567890").unwrap();
        assert_eq!(amount3.0.precision, 6);

        let amount0 = USDCAmount::from_str("roughly 123.456").unwrap();
        assert_eq!(amount0.0.precision, 3);
    }

    #[test]
    fn test_usdc_amount_exact_equality() {
        let amount1 = USDCAmount::from_str("123.456").unwrap();
        let amount2 = USDCAmount::from_str("123.456").unwrap();
        assert_eq!(amount1, amount2);
    }

    #[test]
    fn test_usdc_amount_equality_with_different_precision() {
        // Same value but different string representation
        let amount1 = USDCAmount::from_str("123.4560").unwrap();
        let amount2 = USDCAmount::from_str("123.456").unwrap();
        assert_eq!(amount1, amount2);

        let amount3 = USDCAmount::from_str("123.45").unwrap();
        let amount4 = USDCAmount::from_str("123.450000").unwrap();
        assert_eq!(amount3, amount4);
    }

    #[test]
    fn test_usdc_amount_inequality() {
        let amount3 = USDCAmount::from_str("123.000").unwrap();
        let amount4 = USDCAmount::from_str("123.001").unwrap();
        assert_ne!(amount3, amount4);
    }

    #[test]
    fn test_usdc_amount_parsing_whole_numbers() {
        let amount = USDCAmount::from_str("123").unwrap();
        let expected = USDCAmount::from(U256::from(123_000_000u64)); // 123 * 10^6
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_usdc_amount_parsing_with_decimals() {
        let amount = USDCAmount::from_str("123.456").unwrap();
        // 123.456 = 123_456_000 / 10^6
        let expected = USDCAmount::from(U256::from(123_456_000u64));
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_usdc_amount_parsing_with_fewer_decimals() {
        let amount = USDCAmount::from_str("123.45").unwrap();
        // 123.45 = 123_450_000 / 10^6
        let expected = USDCAmount::from(U256::from(123_450_000u64));
        assert_eq!(amount, expected);
    }

    #[test]
    fn test_usdc_amount_parsing_with_more_decimals() {
        let amount = USDCAmount::from_str("123.4567890").unwrap();
        // For 6 decimal precision, should truncate to 6 decimal places (123.456789)
        let expected = USDCAmount::from(U256::from(123_456_789u64));

        // Since we're using USDCAmount::from to create expected, we're not doing string parsing
        // So amount and expected should have the exact same U256 values inside
        assert_eq!(amount.0.amount, expected.0.amount);
    }

    #[test]
    fn test_usdc_amount_parsing_invalid_format() {
        let result = USDCAmount::from_str("123.456.789");
        assert!(result.is_err());

        let result = USDCAmount::from_str("abc");
        assert!(result.is_err());

        let result = USDCAmount::from_str("123abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_usdc_amount_from_u256() {
        let u256_value = U256::from(123_456_789u64);
        let amount = USDCAmount::from(u256_value);

        // When creating from U256, the value is used directly with precision=DECIMALS
        // For USDC (6 decimals), 123_456_789 should be 123.456789
        // But this would be represented as 123_456_789 internally
        assert_eq!(amount.amount(), u256_value);
    }

    #[test]
    fn test_usdc_amount_comparison() {
        let amount1 = USDCAmount::from_str("100.0").unwrap();
        let amount2 = USDCAmount::from_str("200.0").unwrap();

        assert!(amount1 < amount2);
        assert!(amount2 > amount1);
    }

    #[test]
    fn test_usdc_amount_zero() {
        let amount = USDCAmount::from_str("0").unwrap();
        let expected = USDCAmount::from(U256::from(0u64));
        assert_eq!(amount, expected);

        let amount = USDCAmount::from_str("0.0").unwrap();
        assert_eq!(amount, expected);

        let amount = USDCAmount::from_str("0.000000").unwrap();
        assert_eq!(amount, expected);
    }

    // Add a test for the overflow case that's causing issues
    #[test]
    fn test_parsing_with_longer_than_decimal_precision() {
        // USDC has 6 decimal places, but we provide more
        let result = USDCAmount::from_str("0.1234567890");
        assert!(
            result.is_ok(),
            "Should be able to parse values with more decimal places"
        );

        let amount = result.unwrap();
        // Should truncate to 0.123456
        assert_eq!(amount.amount(), U256::from(123_456u64));
    }

    #[test]
    fn test_one_decimal_place_difference() {
        let amount1 = USDCAmount::from_str("50.00000").unwrap();
        assert_eq!(amount1.0.precision, 6);
        let amount2 = USDCAmount::from_str("49.999999").unwrap();
        assert_eq!(amount2.0.precision, 6);

        assert_ne!(amount1, amount2);

        let amount1 = USDCAmount::from_str("roughly 50").unwrap();
        assert_eq!(amount1, amount2);
    }
}
