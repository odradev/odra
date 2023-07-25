use alloc::string::String;
use odra::{
    contract_env,
    types::{event::OdraEvent, Address, U256,},
    Readable,
    Writeable,
    UnwrapOrRevert
};

use self::{
    errors::Error,
    events::{Approval, Transfer}
};

const DECIMALS: &[u8] = b"decimals";
const SYMBOL: &[u8] = b"symbol";
const TOTAL_SUPPLY: &[u8] = b"total_supply";
const NAME: &[u8] = b"name";
const BALANCES: &[u8] = b"balances";
const ALLOWANCES: &[u8] = b"allowances";

#[odra::module(events = [Approval, Transfer])]
pub struct Erc20; 

#[odra::module]
impl Erc20 {
    #[odra(init)]
    pub fn init(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: &Option<U256>
    ) {
        let caller = contract_env::caller();

        SYMBOL.write(symbol);
        NAME.write(name);
        DECIMALS.write(decimals);

        if let Some(initial_supply) = *initial_supply {
            TOTAL_SUPPLY.write(initial_supply);
            (BALANCES, &caller).write(initial_supply);

            if !initial_supply.is_zero() {
                Transfer {
                    from: None,
                    to: Some(caller),
                    amount: initial_supply
                }
                .emit();
            }
        }
    }

    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        let caller = contract_env::caller();
        self.raw_transfer(&caller, recipient, amount);
    }

    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let spender = contract_env::caller();

        self.spend_allowance(owner, &spender, amount);
        self.raw_transfer(owner, recipient, amount);
    }

    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        let owner = contract_env::caller();
        (ALLOWANCES, &owner, spender).write(*amount);

        Approval {
            owner,
            spender: *spender,
            value: *amount
        }
        .emit();
    }

    pub fn name(&self) -> String {
        NAME.read().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn symbol(&self) -> String {
        SYMBOL.read().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn decimals(&self) -> u8 {
        DECIMALS.read().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn total_supply(&self) -> U256 {
        TOTAL_SUPPLY.read().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn balance_of(&self, address: &Address) -> U256 {
        (BALANCES, address).read().unwrap_or_default()
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        (ALLOWANCES, owner, spender).read().unwrap_or_default()
    }

    pub fn mint(&mut self, address: &Address, amount: &U256) {
        let total_supply: U256 = TOTAL_SUPPLY.read().unwrap_or_default();
        TOTAL_SUPPLY.write(total_supply + *amount);

        let balance: U256 = (BALANCES, address).read().unwrap_or_default();
        (BALANCES, address).write(balance + *amount);

        Transfer {
            from: None,
            to: Some(*address),
            amount: *amount
        }
        .emit();
    }

    pub fn burn(&mut self, address: &Address, amount: &U256) {
        if self.balance_of(address) < *amount {
            contract_env::revert(Error::InsufficientBalance);
        }
        let total_supply: U256 = TOTAL_SUPPLY.read().unwrap_or_default();
        TOTAL_SUPPLY.write(total_supply + *amount);

        let balance: U256 = (BALANCES, address).read().unwrap_or_default();
        (BALANCES, address).write(balance - *amount);

        Transfer {
            from: Some(*address),
            to: None,
            amount: *amount
        }
        .emit();
    }
}

impl Erc20 {
    fn raw_transfer(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let balance: U256 = (BALANCES, owner).read().unwrap_or_default();
        if *amount > balance {
            contract_env::revert(Error::InsufficientBalance)
        }
        (BALANCES, owner).write(balance - *amount);

        let balance: U256 = (BALANCES, recipient).read().unwrap_or_default();
        (BALANCES, recipient).write(balance + *amount);

        Transfer {
            from: Some(*owner),
            to: Some(*recipient),
            amount: *amount
        }
        .emit();
    }

    fn spend_allowance(&mut self, owner: &Address, spender: &Address, amount: &U256) {
        let allowance: U256 = (ALLOWANCES, owner, spender).read().unwrap_or_default();
        if allowance < *amount {
            contract_env::revert(Error::InsufficientAllowance)
        }
        (ALLOWANCES, owner, spender).write(allowance - *amount);

        Approval {
            owner: *owner,
            spender: *spender,
            value: allowance - *amount
        }
        .emit();
    }
}

pub mod events {
    use odra::types::{Address, U256};
    use odra::Event;

    #[derive(Event, Eq, PartialEq, Debug)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub amount: U256
    }

    #[derive(Event, Eq, PartialEq, Debug)]
    pub struct Approval {
        pub owner: Address,
        pub spender: Address,
        pub value: U256
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            InsufficientBalance => 30_000,
            InsufficientAllowance => 30_001,
            NameNotSet => 30_002,
            SymbolNotSet => 30_003,
            DecimalsNotSet => 30_004,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        errors::Error,
        events::{Approval, Transfer},
        Erc20Deployer, Erc20Ref
    };
    use alloc::string::ToString;
    use odra::{assert_events, test_env, types::U256};

    const NAME: &str = "Plascoin";
    const SYMBOL: &str = "PLS";
    const DECIMALS: u8 = 10;
    const INITIAL_SUPPLY: u32 = 10_000;

    fn setup() -> Erc20Ref {
        Erc20Deployer::init(
            SYMBOL.to_string(),
            NAME.to_string(),
            DECIMALS,
            &Some(INITIAL_SUPPLY.into())
        )
    }

    #[test]
    fn initialization() {
        // When deploy a contract with the initial supply.
        let erc20 = setup();

        // Then the contract has the metadata set.
        assert_eq!(erc20.symbol(), SYMBOL.to_string());
        assert_eq!(erc20.name(), NAME.to_string());
        assert_eq!(erc20.decimals(), DECIMALS);

        // Then the total supply is updated.
        assert_eq!(erc20.total_supply(), INITIAL_SUPPLY.into());

        // Then a Transfer event was emitted.
        assert_events!(
            erc20,
            Transfer {
                from: None,
                to: Some(test_env::get_account(0)),
                amount: INITIAL_SUPPLY.into()
            }
        );
    }

    #[test]
    fn transfer_works() {
        // Given a new contract.
        let mut erc20 = setup();

        // When transfer tokens to a recipient.
        let sender = test_env::get_account(0);
        let recipient = test_env::get_account(1);
        let amount = 1_000.into();
        erc20.transfer(&recipient, &amount);

        // Then the sender balance is deducted.
        assert_eq!(
            erc20.balance_of(&sender),
            U256::from(INITIAL_SUPPLY) - amount
        );

        // Then the recipient balance is updated.
        assert_eq!(erc20.balance_of(&recipient), amount);

        // Then Transfer event was emitted.
        assert_events!(
            erc20,
            Transfer {
                from: Some(sender),
                to: Some(recipient),
                amount
            }
        );
    }

    #[test]
    fn transfer_error() {
        test_env::assert_exception(Error::InsufficientBalance, || {
            // Given a new contract.
            let mut erc20 = setup();

            // When the transfer amount exceeds the sender balance.
            let recipient = test_env::get_account(1);
            let amount = U256::from(INITIAL_SUPPLY) + U256::one();

            // Then an error occurs.
            erc20.transfer(&recipient, &amount)
        });
    }

    #[test]
    fn transfer_from_and_approval_work() {
        let mut erc20 = setup();
        let (owner, recipient, spender) = (
            test_env::get_account(0),
            test_env::get_account(1),
            test_env::get_account(2)
        );
        let approved_amount = 3_000.into();
        let transfer_amount = 1_000.into();

        assert_eq!(erc20.balance_of(&owner), U256::from(INITIAL_SUPPLY));

        // Owner approves Spender.
        erc20.approve(&spender, &approved_amount);

        // Allowance was recorded.
        assert_eq!(erc20.allowance(&owner, &spender), approved_amount);
        assert_events!(
            erc20,
            Approval {
                owner,
                spender,
                value: approved_amount
            }
        );

        // Spender transfers tokens from Owner to Recipient.
        test_env::set_caller(spender);
        erc20.transfer_from(&owner, &recipient, &transfer_amount);

        // Tokens are transferred and allowance decremented.
        assert_eq!(
            erc20.balance_of(&owner),
            U256::from(INITIAL_SUPPLY) - transfer_amount
        );
        assert_eq!(erc20.balance_of(&recipient), transfer_amount);
        assert_events!(
            erc20,
            Approval {
                owner,
                spender,
                value: approved_amount - transfer_amount
            },
            Transfer {
                from: Some(owner),
                to: Some(recipient),
                amount: transfer_amount
            }
        );
    }

    #[test]
    fn transfer_from_error() {
        test_env::assert_exception(Error::InsufficientAllowance, || {
            // Given a new instance.
            let mut erc20 = setup();

            // When the spender's allowance is zero.
            let (owner, spender, recipient) = (
                test_env::get_account(0),
                test_env::get_account(1),
                test_env::get_account(2)
            );
            let amount = 1_000.into();
            test_env::set_caller(spender);

            // Then transfer fails.
            erc20.transfer_from(&owner, &recipient, &amount)
        });
    }
}
