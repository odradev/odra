//! This module contains the parameter types used in the cucumber tests.

use cucumber::Parameter;
use odra_core::casper_types::U256;
use std::{fmt::Display, ops::Deref, str::FromStr};
const ONE_CSPR: f64 = 1e9;

/// Account parameter
#[repr(u16)]
#[derive(Debug, Parameter, Clone)]
#[param(name = "account", regex = ".+")]
pub enum Account {
    /// Alice account
    Alice = 1,
    /// Bob account
    Bob = 2,
    /// Charlie account
    Charlie = 3,
    /// Dan account
    Dan = 4,
    /// Eve account
    Eve = 5,
    /// Fred account
    Fred = 6,
    /// George account
    George = 7,
    /// Harry account
    Harry = 8,
    /// Ian account
    Ian = 9,
    /// John account
    John = 10,
    /// A contract account
    ///
    /// A gherkin step can use this account by specifying the account name followed by "Contract".
    Contract(String),
    /// Custom role account
    ///
    /// A gherkin step can use this account by specifying the account name different than the named accounts.
    CustomRole(String)
}

impl FromStr for Account {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if s.ends_with("Contract") {
            return Ok(Account::Contract(s.to_string()));
        }
        match s {
            "Alice" => Ok(Account::Alice),
            "Bob" => Ok(Account::Bob),
            "Charlie" => Ok(Account::Charlie),
            "Dan" => Ok(Account::Dan),
            "Eve" => Ok(Account::Eve),
            "Fred" => Ok(Account::Fred),
            "George" => Ok(Account::George),
            "Harry" => Ok(Account::Harry),
            "Ian" => Ok(Account::Ian),
            "John" => Ok(Account::John),
            _ => Ok(Account::CustomRole(s.to_string()))
        }
    }
}

/// Amount parameter type. It represents the amount of a token.
/// Wraps a `U256` value.
#[derive(Debug, Parameter, Clone, Copy)]
#[param(name = "amount", regex = ".+")]
pub struct Amount(U256);

impl FromStr for Amount {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let num = s.parse::<f64>().expect("Should be a number");
        let num = U256::from((num * ONE_CSPR).round() as u64);
        Ok(Self(num))
    }
}

impl From<U256> for Amount {
    fn from(num: U256) -> Self {
        Self(num)
    }
}

impl From<u64> for Amount {
    fn from(num: u64) -> Self {
        Self(U256::from(num))
    }
}

impl Amount {
    /// Returns the amount as a `f64` value.
    pub fn as_f64(&self) -> f64 {
        let num = self.0.as_u64();
        num as f64 / ONE_CSPR
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_f64())
    }
}

impl Deref for Amount {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Time unit parameter type.
#[derive(Debug, Parameter)]
#[param(name = "time_unit", regex = r".*")]
pub enum TimeUnit {
    /// Seconds unit
    Seconds,
    /// Minutes unit
    Minutes,
    /// Hours unit
    Hours,
    /// Days unit
    Days
}

impl FromStr for TimeUnit {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match s {
            "seconds" | "second" => Self::Seconds,
            "minutes" | "minute" => Self::Minutes,
            "hours" | "hour" => Self::Hours,
            "days" | "day" => Self::Days,
            invalid => {
                panic!("Unknown unit {:?} option - it should be either seconds, minutes, hours or days", invalid)
            }
        })
    }
}

/// Result parameter type.
#[derive(Debug, Parameter)]
#[param(
    name = "result",
    regex = r"succeeds|fails|success|failure|pass|passed|true|fail|failed|false"
)]
pub enum Result {
    /// Success result
    Success,
    /// Failure result
    Failure
}

impl FromStr for Result {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(match s {
            // "succeeds" and synonyms are considered as success, while "fails" and synonyms is considered as failure
            "succeeds" | "success" | "pass" | "passed" | "true" => Self::Success,
            "fails" | "failure" | "fail" | "failed" | "false" => Self::Failure,
            invalid => {
                panic!(
                    "Unknown result {:?} option - it should be either succeeds or fails",
                    invalid
                )
            }
        })
    }
}

impl Deref for Result {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        match self {
            Result::Success => &true,
            Result::Failure => &false
        }
    }
}
