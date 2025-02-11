use cucumber::Parameter;
use std::str::FromStr;

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
            _ => Err(format!("Unknown account: {}", s))
        }
    }
}

impl Account {
    pub fn account_id(&self) -> usize {
        self.account_id
    }
}
