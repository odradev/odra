use cucumber::Parameter;
use odra::casper_types::U256;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Parameter, Debug, Clone, Copy)]
#[param(regex = r"\d+(\.\d+)?", name = "token_amount")]
pub struct TokenAmount {
    amount: U256,
    precision: usize
}

impl PartialEq for TokenAmount {
    fn eq(&self, other: &Self) -> bool {
        let min_precision = self.precision.min(other.precision);

        let tolerance = U256::from(10u64).pow(U256::from(min_precision));
        let diff = if self.amount > other.amount {
            self.amount - other.amount
        } else {
            other.amount - self.amount
        };

        diff <= tolerance
    }
}

impl PartialOrd for TokenAmount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

impl TokenAmount {
    pub fn new(amount: U256, precision: usize) -> Self {
        TokenAmount { amount, precision }
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }
}

impl Deref for TokenAmount {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.amount
    }
}

impl Display for TokenAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl FromStr for TokenAmount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                // No decimal point
                let amount = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                Ok(TokenAmount {
                    amount: amount * U256::from(1_000_000_000u64),
                    precision: 0
                })
            }
            2 => {
                // Has decimal point
                let whole = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                let mut decimal = parts[1].to_string();
                let precision = 9 - decimal.len();
                // Pad with zeros if less than 9 decimal places
                while decimal.len() < 9 {
                    decimal.push('0');
                }
                // Truncate if more than 9 decimal places
                decimal.truncate(9);
                let fractional = U256::from_dec_str(&decimal).map_err(|e| e.to_string())?;

                Ok(TokenAmount {
                    amount: (whole * U256::from(1_000_000_000u64)) + fractional,
                    precision
                })
            }
            _ => Err("Invalid token amount format".to_string())
        }
    }
}

impl From<U256> for TokenAmount {
    fn from(amount: U256) -> Self {
        TokenAmount {
            amount,
            precision: 9
        }
    }
}

impl From<&U256> for TokenAmount {
    fn from(amount: &U256) -> Self {
        TokenAmount {
            amount: *amount,
            precision: 9
        }
    }
}
