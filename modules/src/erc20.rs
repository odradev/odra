use crate::erc20::errors::Error::*;
use crate::erc20::events::*;
use odra::prelude::*;
use odra::{Address, Mapping, Module, Variable, U256};

#[odra::module(events = [Approval, Transfer])]
pub struct Erc20 {
    decimals: Variable<u8>,
    symbol: Variable<String>,
    name: Variable<String>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<(Address, Address), U256>
}

#[odra::module]
impl Erc20 {
    pub fn init(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: Option<U256>
    ) {
        let caller = self.env().caller();
        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);

        if let Some(initial_supply) = initial_supply {
            self.total_supply.set(initial_supply);
            self.balances.set(&caller, initial_supply);

            if !initial_supply.is_zero() {
                self.env().emit_event(Transfer {
                    from: None,
                    to: Some(caller),
                    amount: initial_supply
                });
            }
        }
    }

    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        let caller = self.env().caller();
        self.raw_transfer(&caller, recipient, amount);
    }

    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let spender = self.env().caller();

        self.spend_allowance(owner, &spender, amount);
        self.raw_transfer(owner, recipient, amount);
    }

    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        let owner = self.env().caller();

        self.allowances.set(&(owner, *spender), *amount);
        self.env().emit_event(Approval {
            owner,
            spender: *spender,
            value: *amount
        });
    }

    pub fn name(&self) -> String {
        self.name.get_or_revert_with(NameNotSet)
    }

    pub fn symbol(&self) -> String {
        self.symbol.get_or_revert_with(SymbolNotSet)
    }

    pub fn decimals(&self) -> u8 {
        self.decimals.get_or_revert_with(DecimalsNotSet)
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get_or_default(address)
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances.get_or_default(&(*owner, *spender))
    }

    pub fn mint(&mut self, address: &Address, amount: &U256) {
        self.total_supply.add(*amount);
        self.balances.add(address, *amount);

        self.env().emit_event(Transfer {
            from: None,
            to: Some(*address),
            amount: *amount
        });
    }

    pub fn burn(&mut self, address: &Address, amount: &U256) {
        if self.balance_of(address) < *amount {
            self.env().revert(InsufficientBalance);
        }
        self.total_supply.subtract(*amount);
        self.balances.subtract(address, *amount);

        self.env().emit_event(Transfer {
            from: Some(*address),
            to: None,
            amount: *amount
        });
    }
}

impl Erc20 {
    fn raw_transfer(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        if *amount > self.balances.get_or_default(owner) {
            self.env().revert(InsufficientBalance)
        }

        self.balances.subtract(owner, *amount);
        self.balances.add(recipient, *amount);

        self.env().emit_event(Transfer {
            from: Some(*owner),
            to: Some(*recipient),
            amount: *amount
        });
    }

    fn spend_allowance(&mut self, owner: &Address, spender: &Address, amount: &U256) {
        let allowance = self.allowances.get_or_default(&(*owner, *spender));
        if allowance < *amount {
            self.env().revert(InsufficientAllowance)
        }
        self.allowances.subtract(&(*owner, *spender), *amount);

        self.env().emit_event(Approval {
            owner: *owner,
            spender: *spender,
            value: allowance - *amount
        });
    }
}

pub mod events {
    use casper_event_standard::Event;
    use odra::{Address, U256};

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
    use odra::OdraError;

    #[derive(OdraError)]
    pub enum Error {
        InsufficientBalance = 30_000,
        InsufficientAllowance = 30_001,
        NameNotSet = 30_002,
        SymbolNotSet = 30_003,
        DecimalsNotSet = 30_004
    }
}

#[cfg(test)]
mod tests {
    use super::{
        errors::Error,
        events::{Approval, Transfer},
        Erc20Deployer, Erc20HostRef
    };
    use odra::prelude::*;
    use odra::{HostEnv, U256};

    const NAME: &str = "Plascoin";
    const SYMBOL: &str = "PLS";
    const DECIMALS: u8 = 10;
    const INITIAL_SUPPLY: u32 = 10_000;

    fn setup() -> (HostEnv, Erc20HostRef) {
        let env = odra_test::env();
        (
            env.clone(),
            Erc20Deployer::init(
                &env,
                SYMBOL.to_string(),
                NAME.to_string(),
                DECIMALS,
                Some(INITIAL_SUPPLY.into())
            )
        )
    }

    #[test]
    fn initialization() {
        // When deploy a contract with the initial supply.
        let (env, erc20) = setup();

        // Then the contract has the metadata set.
        assert_eq!(erc20.symbol(), SYMBOL.to_string());
        assert_eq!(erc20.name(), NAME.to_string());
        assert_eq!(erc20.decimals(), DECIMALS);

        // Then the total supply is updated.
        assert_eq!(erc20.total_supply(), INITIAL_SUPPLY.into());

        // Then a Transfer event was emitted.
        assert!(env.emitted_event(
            erc20.address(),
            &Transfer {
                from: None,
                to: Some(env.get_account(0)),
                amount: INITIAL_SUPPLY.into()
            }
        ));
    }

    #[test]
    fn transfer_works() {
        // Given a new contract.
        let (env, mut erc20) = setup();

        // When transfer tokens to a recipient.
        let sender = env.get_account(0);
        let recipient = env.get_account(1);
        let amount = 1_000.into();
        erc20.transfer(recipient, amount);

        // Then the sender balance is deducted.
        assert_eq!(
            erc20.balance_of(sender),
            U256::from(INITIAL_SUPPLY) - amount
        );

        // Then the recipient balance is updated.
        assert_eq!(erc20.balance_of(recipient), amount);

        // Then Transfer event was emitted.
        assert!(env.emitted_event(
            erc20.address(),
            &Transfer {
                from: Some(sender),
                to: Some(recipient),
                amount
            }
        ));
    }

    #[test]
    fn transfer_error() {
        // Given a new contract.
        let (env, mut erc20) = setup();

        // When the transfer amount exceeds the sender balance.
        let recipient = env.get_account(1);
        let amount = U256::from(INITIAL_SUPPLY) + U256::one();

        // Then an error occurs.
        assert!(erc20.try_transfer(recipient, amount).is_err());
    }

    #[test]
    fn transfer_from_and_approval_work() {
        let (env, mut erc20) = setup();

        let (owner, recipient, spender) =
            (env.get_account(0), env.get_account(1), env.get_account(2));
        let approved_amount = 3_000.into();
        let transfer_amount = 1_000.into();

        assert_eq!(erc20.balance_of(owner), U256::from(INITIAL_SUPPLY));

        // Owner approves Spender.
        erc20.approve(spender, approved_amount);

        // Allowance was recorded.
        assert_eq!(erc20.allowance(owner, spender), approved_amount);
        assert!(env.emitted_event(
            erc20.address(),
            &Approval {
                owner,
                spender,
                value: approved_amount
            }
        ));

        // Spender transfers tokens from Owner to Recipient.
        env.set_caller(spender);
        erc20.transfer_from(owner, recipient, transfer_amount);

        // Tokens are transferred and allowance decremented.
        assert_eq!(
            erc20.balance_of(owner),
            U256::from(INITIAL_SUPPLY) - transfer_amount
        );
        assert_eq!(erc20.balance_of(recipient), transfer_amount);
        assert!(env.emitted_event(
            erc20.address(),
            &Approval {
                owner,
                spender,
                value: approved_amount - transfer_amount
            }
        ));
        assert!(env.emitted_event(
            erc20.address(),
            &Transfer {
                from: Some(owner),
                to: Some(recipient),
                amount: transfer_amount
            }
        ));
    }

    #[test]
    fn transfer_from_error() {
        // Given a new instance.
        let (env, mut erc20) = setup();

        // When the spender's allowance is zero.
        let (owner, spender, recipient) =
            (env.get_account(0), env.get_account(1), env.get_account(2));
        let amount = 1_000.into();
        env.set_caller(spender);

        // Then transfer fails.
        assert_eq!(
            erc20.try_transfer_from(owner, recipient, amount),
            Err(Error::InsufficientAllowance.into())
        );
    }
}
