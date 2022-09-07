use odra_env::ContractEnv;
use odra_types::{event::Event, Address, U256};

use crate::{erc20, erc20::Erc20};

use self::events::{Deposit, Withdrawal};

#[odra_proc_macros::module]
pub struct WrappedNativeToken {
    erc20: Erc20,
}

#[odra_proc_macros::module]
impl WrappedNativeToken {
    #[odra(init)]
    pub fn init(&self) {
        let token = ContractEnv::native_token();
        self.erc20
            .init(token.symbol(), token.name(), token.decimals());
    }

    //payable
    pub fn deposit(&self, amount: U256) {
        let caller = ContractEnv::caller();
        erc20::ext::mint(&self.erc20, caller, amount);

        ContractEnv::deposit_native_token(caller, amount);

        Deposit {
            account: caller,
            value: amount,
        }
        .emit();
    }

    //payable
    pub fn withdraw(&self, amount: U256) {
        let caller = ContractEnv::caller();
        erc20::ext::burn(&self.erc20, caller, amount);

        ContractEnv::withdraw_native_token(caller, amount);

        Withdrawal {
            account: caller,
            value: amount,
        }
        .emit()
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    pub fn balance_of(&self, address: Address) -> U256 {
        self.erc20.balance_of(address)
    }

    pub fn total_supply(&self) -> U256 {
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

    pub fn approve(&self, spender: Address, amount: U256) {
        self.erc20.approve(spender, amount)
    }

    pub fn transfer_from(&self, owner: Address, recipient: Address, amount: U256) {
        self.erc20.transfer_from(owner, recipient, amount)
    }

    pub fn transfer(&self, recipient: Address, amount: U256) {
        self.erc20.transfer(recipient, amount)
    }
}

pub mod events {
    use odra_proc_macros::Event;
    use odra_types::{Address, U256};

    #[derive(Event, Debug, PartialEq)]
    pub struct Deposit {
        pub account: Address,
        pub value: U256,
    }

    #[derive(Event, Debug, PartialEq)]
    pub struct Withdrawal {
        pub account: Address,
        pub value: U256,
    }
}

#[cfg(all(test, feature = "mock-vm"))]
mod tests {

    use odra_env::{assert_events, ContractEnv, TestEnv};
    use odra_types::{Address, OdraError, VmError, U256};

    use crate::{
        erc20::events::Transfer,
        wrapped_native::events::{Deposit, Withdrawal},
    };

    use super::{WrappedNativeToken, WrappedNativeTokenRef};

    fn setup() -> (WrappedNativeTokenRef, Address, U256, Address, U256) {
        let token: WrappedNativeTokenRef = WrappedNativeToken::deploy_init();
        let account_1 = TestEnv::get_account(0);
        let account_1_balance = TestEnv::get_balance(account_1);
        let account_2 = TestEnv::get_account(1);
        let account_2_balance = TestEnv::get_balance(account_2);

        (
            token,
            account_1,
            account_1_balance,
            account_2,
            account_2_balance,
        )
    }

    #[test]
    fn test_init() {
        let token: WrappedNativeTokenRef = WrappedNativeToken::deploy_init();

        let native_token = ContractEnv::native_token();

        assert_eq!(token.total_supply(), U256::zero());
        assert_eq!(token.name(), native_token.name());
        assert_eq!(token.symbol(), native_token.symbol());
        assert_eq!(token.decimals(), native_token.decimals());
    }

    #[test]
    fn test_deposit() {
        let (token, account, account_balance, _, _) = setup();

        let amount: U256 = 1_000.into();

        token.deposit(amount);

        assert_eq!(token.balance_of(account), amount);
        assert_eq!(TestEnv::get_balance(account), account_balance - amount);

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

        let amount: U256 = 1_000.into();

        TestEnv::set_caller(&account_1);
        token.deposit(amount);

        TestEnv::set_caller(&account_2);
        token.deposit(amount);

        assert_eq!(token.total_supply(), amount + amount);
        assert_events!(token, Transfer, Deposit, Transfer, Deposit);
    }

    #[test]
    fn test_deposit_amount_exceeding_account_balance() {
        let (token, _, balance, _, _) = setup();

        //TODO: Consider what really should happen here.
        TestEnv::assert_exception(OdraError::VmError(VmError::Panic), || {
            token.deposit(balance + U256::one());
        });
    }

    #[test]
    fn test_withdrawal() {
        let (token, account, balance, _, _) = setup();

        let amount = 1_000.into();
        token.deposit(amount);

        let withdrawal_amount = 100.into();
        token.withdraw(withdrawal_amount);

        let native_token_balance_diff = amount - withdrawal_amount;
        assert_eq!(
            TestEnv::get_balance(account),
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
        TestEnv::assert_exception(crate::erc20::errors::Error::InsufficientBalance, || {
            token.withdraw(U256::one());
        });
    }
}
