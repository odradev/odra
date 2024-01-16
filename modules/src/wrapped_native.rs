use crate::erc20::Erc20;
use crate::wrapped_native::events::{Deposit, Withdrawal};
use odra::prelude::*;
use odra::uints::{ToU256, ToU512};
use odra::{Address, Module, ModuleWrapper, UnwrapOrRevert, U256};

#[odra::module(events = [Deposit, Withdrawal])]
pub struct WrappedNativeToken {
    erc20: ModuleWrapper<Erc20>
}

#[odra::module]
impl WrappedNativeToken {
    pub fn init(&mut self) {
        let symbol = "WCSPR".to_string();
        let name = "Wrapped CSPR".to_string();
        self.erc20.init(symbol, name, 9, None);
    }

    #[odra(payable)]
    pub fn deposit(&mut self) {
        let caller = self.env().caller();

        let amount = self.env().attached_value();

        let amount = amount.to_u256().unwrap_or_revert(&self.env());
        self.erc20.mint(&caller, &amount);

        self.env().emit_event(Deposit {
            account: caller,
            value: amount
        });
    }

    pub fn withdraw(&mut self, amount: &U256) {
        let caller = self.env().caller();

        self.erc20.burn(&caller, amount);
        self.env().transfer_tokens(&caller, &amount.to_u512());

        self.env().emit_event(Withdrawal {
            account: caller,
            value: *amount
        });
    }

    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    pub fn balance_of(&self, address: &Address) -> U256 {
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

    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        self.erc20.approve(spender, amount)
    }

    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        self.erc20.transfer_from(owner, recipient, amount)
    }

    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        self.erc20.transfer(recipient, amount)
    }
}

pub mod events {
    use casper_event_standard::Event;
    use odra::{Address, U256};

    #[derive(Event, Debug, Eq, PartialEq)]
    pub struct Deposit {
        pub account: Address,
        pub value: U256
    }

    #[derive(Event, Debug, Eq, PartialEq)]
    pub struct Withdrawal {
        pub account: Address,
        pub value: U256
    }
}

#[cfg(test)]
mod tests {
    use crate::erc20::errors::Error::InsufficientBalance;
    use crate::erc20::events::Transfer;
    use crate::wrapped_native::events::{Deposit, Withdrawal};
    use crate::wrapped_native::{WrappedNativeTokenDeployer, WrappedNativeTokenHostRef};
    use casper_event_standard::EventInstance;
    use odra::prelude::*;
    use odra::uints::{ToU256, ToU512};
    use odra::VmError::BalanceExceeded;
    use odra::{Address, HostEnv, OdraError, U256, U512};

    fn setup() -> (
        HostEnv,
        WrappedNativeTokenHostRef,
        Address,
        U512,
        Address,
        U512
    ) {
        let env = odra_test::test_env();
        let token = WrappedNativeTokenDeployer::init(&env);
        let account_1 = env.get_account(0);
        let account_1_balance = env.balance_of(&account_1);
        let account_2 = env.get_account(1);
        let account_2_balance = env.balance_of(&account_2);

        (
            env,
            token,
            account_1,
            account_1_balance,
            account_2,
            account_2_balance
        )
    }

    #[test]
    fn test_init() {
        // When deploy a contract.
        let (_, token, _, _, _, _) = setup();

        // Then the contract has correct metadata.
        assert_eq!(token.total_supply(), U256::zero());
        assert_eq!(token.name(), "Wrapped CSPR".to_string());
        assert_eq!(token.symbol(), "WCSPR".to_string());
        assert_eq!(token.decimals(), 9);
    }

    #[test]
    fn test_deposit() {
        // Given a fresh contract.
        let (env, token, account, account_balance, _, _) = setup();

        // When deposit tokens.
        let deposit_amount = 1_000u32;
        token.with_tokens(deposit_amount.into()).deposit();

        // Then native tokens are correctly deducted.
        assert_eq!(account_balance - deposit_amount, env.balance_of(&account));

        // Then the contract balance is updated.
        assert_eq!(token.balance_of(account), deposit_amount.into());

        // The events were emitted.
        assert!(env.emitted_event(
            token.address(),
            &Transfer {
                from: None,
                to: Some(account),
                amount: deposit_amount.into()
            }
        ));

        assert!(env.emitted_event(
            token.address(),
            &Deposit {
                account,
                value: deposit_amount.into()
            }
        ));
    }

    #[test]
    fn test_minting() {
        // Given a fresh contract.
        let (env, token, account_1, _, account_2, _) = setup();

        // When two users deposit some tokens.
        let deposit_amount = 1_000.into();

        env.set_caller(account_1);
        token.with_tokens(deposit_amount).deposit();

        env.set_caller(account_2);
        token.with_tokens(deposit_amount).deposit();

        // Then the total supply in the sum of deposits.
        assert_eq!(
            token.total_supply(),
            (deposit_amount + deposit_amount).to_u256().unwrap()
        );
        // Then events were emitted.
        assert!(env.event_names(token.address()).ends_with(
            vec![
                Transfer::name(),
                Deposit::name(),
                Transfer::name(),
                Deposit::name()
            ]
            .as_slice()
        ));
    }

    #[test]
    fn test_deposit_amount_exceeding_account_balance() {
        // Given a new contract.
        let (_, token, _, balance, _, _) = setup();
        // When the deposit amount exceeds the user's balance
        // Then an error occurs.
        assert_eq!(
            token.with_tokens(balance + U512::one()).try_deposit(),
            Err(OdraError::VmError(BalanceExceeded))
        );
    }

    #[test]
    fn test_withdrawal() {
        // Deposit some tokens in the contract.
        let (env, mut token, account, _, _, _) = setup();
        let deposit_amount: U512 = 3_000.into();
        token.with_tokens(deposit_amount).deposit();
        let account_balance = env.balance_of(&account);

        // When withdraw some tokens.
        let withdrawal_amount: U256 = 1_000.into();
        token.withdraw(withdrawal_amount);

        // Then the user has the withdrawn tokens back.
        assert_eq!(
            account_balance + withdrawal_amount.to_u512(),
            env.balance_of(&account)
        );
        // Then the balance in the contract is deducted.
        assert_eq!(
            token.balance_of(account),
            deposit_amount.to_u256().unwrap() - withdrawal_amount
        );

        // Then events were emitted.
        assert!(env.emitted_event(
            token.address(),
            &Transfer {
                from: Some(account),
                to: None,
                amount: withdrawal_amount
            }
        ));
        assert!(env.emitted_event(
            token.address(),
            &Withdrawal {
                account,
                value: withdrawal_amount
            }
        ));
    }

    #[test]
    fn test_withdrawal_amount_exceeding_account_deposit() {
        // Given a new contract.
        let (_, mut token, _, _, _, _) = setup();
        // When the user withdraws amount exceeds the user's deposit
        // Then an error occurs.
        assert_eq!(
            token.try_withdraw(U256::one()),
            Err(InsufficientBalance.into())
        );
    }
}
