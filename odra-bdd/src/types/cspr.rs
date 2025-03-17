// TODO: CSPRParam

use crate::types::token_amount::TokenAmount;
use cucumber::Parameter;
use odra::casper_types::U512;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Parameter, Debug, Clone, Copy)]
#[param(regex = r"(?:roughly\s+)?\d+(\.\d+)?( Motes)?", name = "cspr_amount")]
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

    pub fn precision(&self) -> usize {
        self.precision
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

        let amount = TokenAmount::<9>::from_str(s);
        // TODO: make from_str generic over U256 and U512
        Ok(CSPRAmount {
            amount: amount.clone()?.amount().as_u128().into(),
            precision: amount?.precision()
        })
    }
}
