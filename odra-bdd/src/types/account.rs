use cucumber::Parameter;
use std::{fmt::Display, str::FromStr};

#[derive(Parameter, PartialEq)]
#[param(regex = r"[A-Za-z]+", name = "account")]
pub struct Account {
    account_id: usize
}

impl FromStr for Account {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "Owner" => Ok(Account { account_id: 0 }),
            "Alice" => Ok(Account { account_id: 1 }),
            "Bob" => Ok(Account { account_id: 2 }),
            "Charlie" => Ok(Account { account_id: 3 }),
            "Dave" => Ok(Account { account_id: 4 }),
            "Eve" => Ok(Account { account_id: 5 }),
            _ => Err(format!("Unknown account: {}", s))
        }
    }
}

impl Account {
    pub fn account_id(&self) -> usize {
        self.account_id
    }

    pub const ALICE: Account = Account { account_id: 1 };
    pub const BOB: Account = Account { account_id: 2 };
    pub const CHARLIE: Account = Account { account_id: 3 };
    pub const DAVE: Account = Account { account_id: 4 };
    pub const EVE: Account = Account { account_id: 5 };
    pub const OWNER: Account = Account { account_id: 0 };
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.account_id {
                0 => "Owner",
                1 => "Alice",
                2 => "Bob",
                3 => "Charlie",
                4 => "Dave",
                5 => "Eve",
                _ => "Unknown"
            }
        )
    }
}
