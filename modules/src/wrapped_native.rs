//! Wrapped CSPR token implementation
use crate::cep18_token::Cep18;
use crate::wrapped_native::events::{Deposit, Withdrawal};
use odra::casper_types::{U256, U512};
use odra::uints::{ToU256, ToU512};
use odra::{prelude::*, ContractRef};

/// An event emitted when native tokens are deposited into the contract.
#[odra::event]
pub struct OnCsprDeposit {
    /// Address of the account that deposited the tokens.
    pub account: Address,
    /// The amount of tokens deposited.
    pub value: U512
}

/// The CsprDeposit contract.
#[odra::module]
pub struct CsprDeposit {}

/// The CsprDeposit contract implementation.
#[odra::module]
impl CsprDeposit {
    /// Deposits native tokens into the contract.
    #[odra(payable)]
    pub fn deposit(&self) {
        self.env().emit_event(OnCsprDeposit {
            account: self.env().caller(),
            value: self.env().attached_value()
        });
    }
}

/// The WrappedNativeToken module.
#[odra::module(events = [Deposit, Withdrawal])]
pub struct WrappedNativeToken {
    token: SubModule<Cep18>
}

/// The WrappedNativeToken module implementation.
#[odra::module]
impl WrappedNativeToken {
    /// Initializes the contract with the metadata.
    pub fn init(&mut self) {
        let symbol = "WCSPR".to_string();
        let name = "Wrapped CSPR".to_string();
        self.token.init(symbol, name, 9, U256::zero());
    }

    /// Deposits native tokens into the contract.
    #[odra(payable)]
    pub fn deposit(&mut self) {
        let caller = self.env().caller();

        let amount = self.env().attached_value();

        let amount = amount.to_u256().unwrap_or_revert(self);
        self.token.raw_mint(&caller, &amount);

        self.env().emit_event(Deposit {
            account: caller,
            value: amount
        });
    }

    /// Withdraws native tokens from the contract.
    pub fn withdraw(&mut self, amount: &U256) {
        let caller = self.env().caller();

        self.token.raw_burn(&caller, amount);
        if caller.is_contract() {
            CsprDepositContractRef::new(self.env(), caller)
                .with_tokens(amount.to_u512())
                .deposit();
        } else {
            self.env().transfer_tokens(&caller, &amount.to_u512());
        }

        self.env().emit_event(Withdrawal {
            account: caller,
            value: *amount
        });
    }

    /// Sets the allowance for `spender` to spend `amount` of the caller's tokens.
    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.token.allowance(owner, spender)
    }

    /// Returns the balance of `address`.
    pub fn balance_of(&self, address: &Address) -> U256 {
        self.token.balance_of(address)
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.token.total_supply()
    }

    /// Returns the number of decimals used by the token.
    pub fn decimals(&self) -> u8 {
        self.token.decimals()
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        self.token.symbol()
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        self.token.name()
    }

    /// Approves `spender` to spend `amount` of the caller's tokens.
    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        self.token.approve(spender, amount)
    }

    /// Transfers `amount` of the owners tokens to `recipient` using allowance.
    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        self.token.transfer_from(owner, recipient, amount)
    }

    /// Transfers `amount` of the caller's tokens to `recipient`.
    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        self.token.transfer(recipient, amount)
    }
}

/// Events emitted by the WrappedNativeToken module.
pub mod events {
    use odra::casper_event_standard;
    use odra::casper_types::U256;
    use odra::prelude::*;

    /// Event emitted when native tokens are deposited into the contract.
    #[odra::event]
    pub struct Deposit {
        /// An Address of the account that deposited the tokens.
        pub account: Address,
        /// The amount of tokens deposited.
        pub value: U256
    }

    /// Event emitted when native tokens are withdrawn from the contract.
    #[odra::event]
    pub struct Withdrawal {
        /// An Address of the account that withdrew the tokens.
        pub account: Address,
        /// The amount of tokens withdrawn.
        pub value: U256
    }
}

#[cfg(test)]
mod tests {
    use crate::cep18::errors::Error::InsufficientBalance;
    use crate::cep18::events::{Burn, Mint};
    use crate::wrapped_native::events::{Deposit, Withdrawal};
    use crate::wrapped_native::WrappedNativeTokenHostRef;
    use odra::casper_event_standard::EventInstance;
    use odra::casper_types::{U256, U512};
    use odra::host::{Deployer, HostEnv, HostRef, NoArgs};
    use odra::prelude::*;
    use odra::uints::{ToU256, ToU512};
    use odra::VmError::BalanceExceeded;

    use super::WrappedNativeToken;

    fn setup() -> (
        HostEnv,
        WrappedNativeTokenHostRef,
        Address,
        U512,
        Address,
        U512
    ) {
        let env = odra_test::env();
        let token = WrappedNativeToken::deploy(&env, NoArgs);
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
        assert_eq!(token.balance_of(&account), deposit_amount.into());

        // The events were emitted.
        assert!(env.emitted_event(
            token.address(),
            &Mint {
                recipient: account,
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
            vec![Mint::name(), Deposit::name(), Mint::name(), Deposit::name()].as_slice()
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
        token.withdraw(&withdrawal_amount);

        // Then the user has the withdrawn tokens back.
        assert_eq!(
            account_balance + withdrawal_amount.to_u512(),
            env.balance_of(&account)
        );
        // Then the balance in the contract is deducted.
        assert_eq!(
            token.balance_of(&account),
            deposit_amount.to_u256().unwrap() - withdrawal_amount
        );

        // Then events were emitted.
        assert!(env.emitted_event(
            token.address(),
            &Burn {
                owner: account,
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
            token.try_withdraw(&U256::one()),
            Err(InsufficientBalance.into())
        );
    }
}
