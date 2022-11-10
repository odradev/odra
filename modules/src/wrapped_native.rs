use odra::{
    contract_env,
    types::{event::OdraEvent, Address, Balance}
};

use crate::{
    erc20::Erc20,
    traits::{Burnable, Mintable}
};

use self::events::{Deposit, Withdrawal};

#[odra::module]
pub struct WrappedNativeToken {
    erc20: Erc20
}

#[odra::module(delegate = [erc20])]
impl WrappedNativeToken {
    #[odra(init)]
    pub fn init(&self) {
        let (name, symbol, decimals) = contract_env::native_token_metadata();
        self.erc20.init(symbol, name, decimals);
    }

    #[odra(payable)]
    pub fn deposit(&self) {
        let caller = contract_env::caller();
        let amount = contract_env::attached_value();

        self.erc20.mint(caller, amount);

        Deposit {
            account: caller,
            value: amount
        }
        .emit();
    }

    pub fn withdraw(&self, amount: Balance) {
        let caller = contract_env::caller();
        self.erc20.burn(caller, amount);

        contract_env::transfer_tokens(caller, amount);

        Withdrawal {
            account: caller,
            value: amount
        }
        .emit()
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Balance {
        self.erc20.allowance(owner, spender)
    }

    pub fn balance_of(&self, address: Address) -> Balance {
        self.erc20.balance_of(address)
    }

    pub fn total_supply(&self) -> Balance {
        self.erc20.total_supply()
    }

    pub fn decimals(&self) -> u8 {
        self.erc20.decimals()
    }

    pub fn symbol(&self) -> String {
        self.erc20.symbol()
    }

    pub fn name(&self) -> String {
        self.erc20.name()
    }

    pub fn approve(&self, spender: Address, amount: Balance) {
        self.erc20.approve(spender, amount)
    }

    pub fn transfer_from(&self, owner: Address, recipient: Address, amount: Balance) {
        self.erc20.transfer_from(owner, recipient, amount)
    }

    pub fn transfer(&self, recipient: Address, amount: Balance) {
        self.erc20.transfer(recipient, amount)
    }
}

pub mod events {
    use odra::{
        types::{Address, Balance},
        Event
    };

    #[derive(Event, Debug, Eq, PartialEq)]
    pub struct Deposit {
        pub account: Address,
        pub value: Balance
    }

    #[derive(Event, Debug, Eq, PartialEq)]
    pub struct Withdrawal {
        pub account: Address,
        pub value: Balance
    }
}

#[cfg(all(test, feature = "mock-vm"))]
mod tests {

    use odra::{
        assert_events, test_env,
        types::{Address, Balance, OdraError, VmError}
    };

    use crate::{
        erc20::events::Transfer,
        wrapped_native::events::{Deposit, Withdrawal}
    };

    use super::{WrappedNativeToken, WrappedNativeTokenRef};

    fn setup() -> (WrappedNativeTokenRef, Address, Balance, Address, Balance) {
        let token: WrappedNativeTokenRef = WrappedNativeToken::deploy_init();
        let account_1 = test_env::get_account(0);
        let account_1_balance = test_env::token_balance(account_1);
        let account_2 = test_env::get_account(1);
        let account_2_balance = test_env::token_balance(account_2);

        (
            token,
            account_1,
            account_1_balance,
            account_2,
            account_2_balance
        )
    }

    #[test]
    fn test_init() {
        let token: WrappedNativeTokenRef = WrappedNativeToken::deploy_init();

        let (name, symbol, decimals) = test_env::native_token_metadata();

        assert_eq!(token.total_supply(), Balance::zero());
        assert_eq!(token.name(), name);
        assert_eq!(token.symbol(), symbol);
        assert_eq!(token.decimals(), decimals);
    }

    #[test]
    fn test_deposit() {
        let (token, account, account_balance, _, _) = setup();

        let amount: Balance = 1_000.into();

        token.with_tokens(amount).deposit();

        assert_eq!(token.balance_of(account), amount);
        assert_eq!(test_env::token_balance(account), account_balance - amount);

        assert_events!(
            token,
            Transfer {
                from: None,
                to: Some(account),
                amount
            },
            Deposit {
                account,
                value: amount
            }
        );
    }

    #[test]
    fn test_minting() {
        let (token, account_1, _, account_2, _) = setup();

        let amount = 1_000u32;

        test_env::set_caller(account_1);
        token.with_tokens(amount).deposit();

        test_env::set_caller(account_2);
        token.with_tokens(amount).deposit();

        assert_eq!(token.total_supply(), (amount + amount).into());
        assert_events!(token, Transfer, Deposit, Transfer, Deposit);
    }

    #[test]
    fn test_deposit_amount_exceeding_account_balance() {
        let (token, _, balance, _, _) = setup();

        //TODO: Consider what really should happen here.
        test_env::assert_exception(OdraError::VmError(VmError::BalanceExceeded), || {
            token.with_tokens(balance + Balance::one()).deposit();
        });
    }

    #[test]
    fn test_withdrawal() {
        let (token, account, balance, _, _) = setup();

        let amount: Balance = 1_000.into();
        token.with_tokens(amount).deposit();

        let withdrawal_amount: Balance = 100.into();
        token.withdraw(withdrawal_amount);

        let native_token_balance_diff = amount - withdrawal_amount;
        assert_eq!(
            test_env::token_balance(account),
            balance - native_token_balance_diff
        );
        assert_eq!(token.balance_of(account), native_token_balance_diff);
        assert_events!(
            token,
            Transfer {
                from: Some(account),
                to: None,
                amount: withdrawal_amount
            },
            Withdrawal {
                account,
                value: withdrawal_amount
            }
        );
    }

    #[test]
    fn test_withdrawal_amount_exceeding_account_deposit() {
        let (token, _, _, _, _) = setup();

        //TODO: Consider what really should happen here.
        test_env::assert_exception(crate::erc20::errors::Error::InsufficientBalance, || {
            token.withdraw(Balance::one());
        });
    }
}
