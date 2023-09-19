use odra::prelude::string::String;
use odra::{
    contract_env,
    types::{casper_types::U256, event::OdraEvent, Address},
    Mapping, UnwrapOrRevert, Variable
};

use self::{
    errors::Error,
    events::{Approval, Transfer}
};

#[odra::module(events = [Approval, Transfer])]
pub struct Erc20 {
    decimals: Variable<u8>,
    symbol: Variable<String>,
    name: Variable<String>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<Address, Mapping<Address, U256>>
}

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

        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);

        if let Some(initial_supply) = *initial_supply {
            self.total_supply.set(initial_supply);
            self.balances.set(&caller, initial_supply);

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

        self.allowances.get_instance(&owner).set(spender, *amount);
        Approval {
            owner,
            spender: *spender,
            value: *amount
        }
        .emit();
    }

    pub fn name(&self) -> String {
        self.name.get().unwrap_or_revert_with(Error::NameNotSet)
    }

    pub fn symbol(&self) -> String {
        self.symbol.get().unwrap_or_revert_with(Error::SymbolNotSet)
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
            .get()
            .unwrap_or_revert_with(Error::DecimalsNotSet)
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get_or_default(address)
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances.get_instance(owner).get_or_default(spender)
    }

    pub fn mint(&mut self, address: &Address, amount: &U256) {
        self.total_supply.add(*amount);
        self.balances.add(address, *amount);

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
        self.total_supply.subtract(*amount);
        self.balances.subtract(address, *amount);

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
        if *amount > self.balances.get_or_default(owner) {
            contract_env::revert(Error::InsufficientBalance)
        }

        self.balances.subtract(owner, *amount);
        self.balances.add(recipient, *amount);

        Transfer {
            from: Some(*owner),
            to: Some(*recipient),
            amount: *amount
        }
        .emit();
    }

    fn spend_allowance(&mut self, owner: &Address, spender: &Address, amount: &U256) {
        let allowance = self.allowances.get_instance(owner).get_or_default(spender);
        if allowance < *amount {
            contract_env::revert(Error::InsufficientAllowance)
        }
        self.allowances
            .get_instance(owner)
            .subtract(spender, *amount);
        Approval {
            owner: *owner,
            spender: *spender,
            value: allowance - *amount
        }
        .emit();
    }
}

pub mod events {
    use odra::types::{casper_types::U256, Address};
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
    use odra::prelude::string::ToString;
    use odra::{assert_events, test_env, types::casper_types::U256};

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
