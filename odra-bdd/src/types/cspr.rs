// TODO: CSPRParam

use cucumber::Parameter;
use odra::casper_types::U512;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Parameter, Debug, Clone, Copy)]
#[param(regex = r"\d+(\.\d+)?( Motes)?", name = "cspr_amount")]
pub struct CSPRAmount {
    amount: U512,
    precision: usize
}

impl PartialEq for CSPRAmount {
    fn eq(&self, other: &Self) -> bool {
        let min_precision = self.precision.min(other.precision);

        let tolerance = U512::from(10u64).pow(U512::from(min_precision));
        let diff = self.amount.abs_diff(other.amount);

        diff <= tolerance
    }
}

impl PartialOrd for CSPRAmount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.amount.partial_cmp(&other.amount)
    }
}

impl CSPRAmount {
    pub fn new(amount: U512, precision: usize) -> Self {
        CSPRAmount { amount, precision }
    }

    pub fn amount(&self) -> U512 {
        self.amount
    }
}

impl Deref for CSPRAmount {
    type Target = U512;

    fn deref(&self) -> &Self::Target {
        &self.amount
    }
}

impl Display for CSPRAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl FromStr for CSPRAmount {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(motes_str) = s.strip_suffix(" Motes") {
            let amount = U512::from_dec_str(motes_str).map_err(|e| e.to_string())?;
            return Ok(CSPRAmount {
                amount,
                precision: 9
            });
        }

        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                let amount = U512::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                Ok(CSPRAmount {
                    amount: amount * U512::from(1_000_000_000u64),
                    precision: 0
                })
            }
            2 => {
                let whole = U512::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                let mut decimal = parts[1].to_string();
                let precision = 9 - decimal.len();
                while decimal.len() < 9 {
                    decimal.push('0');
                }
                decimal.truncate(9);
                let fractional = U512::from_dec_str(&decimal).map_err(|e| e.to_string())?;

                Ok(CSPRAmount {
                    amount: (whole * U512::from(1_000_000_000u64)) + fractional,
                    precision
                })
            }
            _ => Err("Invalid CSPR amount format".to_string())
        }
    }
}
