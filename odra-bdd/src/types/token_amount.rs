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
        let parts: Vec<&str> = s.split('.').collect();
        match parts.len() {
            1 => {
                // No decimal point
                let amount = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                Ok(TokenAmount {
                    amount: amount * Self::multiplier(),
                    precision: 0,
                    _phantom: PhantomData
                })
            }
            2 => {
                // Has decimal point
                let whole = U256::from_dec_str(parts[0]).map_err(|e| e.to_string())?;
                let mut decimal = parts[1].to_string();
                let precision = DECIMALS - decimal.len();
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

#[derive(Debug, Clone, Copy, Parameter)]
#[param(regex = r"\d+(\.\d+)?", name = "usdc")]
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

#[derive(Debug, Clone, Copy, Parameter)]
#[param(regex = r"\d+(\.\d+)?", name = "wcspr")]
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
