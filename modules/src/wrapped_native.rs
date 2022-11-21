use self::events::{Deposit, Withdrawal};
use crate::erc20::Erc20;
use odra::{
    contract_env,
    types::{event::OdraEvent, Address, U256},
    UnwrapOrRevert
};

#[odra::module]
pub struct WrappedNativeToken {
    erc20: Erc20
}

#[odra::module]
impl WrappedNativeToken {
    #[odra(init)]
    pub fn init(&mut self) {
        let metadata = contract_env::native_token_metadata();
        let symbol = format!("W{}", metadata.symbol);
        let name = format!("Wrapped {}", metadata.name);
        self.erc20.init(symbol, name, metadata.decimals, None);
    }

    #[odra(payable)]
    pub fn deposit(&mut self) {
        let caller = contract_env::caller();
        let amount = contract_env::attached_value().to_u256().unwrap_or_revert();

        self.erc20.mint(caller, amount);

        Deposit {
            account: caller,
            value: amount
        }
        .emit();
    }

    pub fn withdraw(&mut self, amount: U256) {
        let caller = contract_env::caller();

        self.erc20.burn(caller, amount);
        let balance = amount.to_balance().unwrap_or_revert();
        contract_env::transfer_tokens(caller, balance);

        Withdrawal {
            account: caller,
            value: amount
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

    pub fn approve(&mut self, spender: Address, amount: U256) {
        self.erc20.approve(spender, amount)
    }

    pub fn transfer_from(&mut self, owner: Address, recipient: Address, amount: U256) {
        self.erc20.transfer_from(owner, recipient, amount)
    }

    pub fn transfer(&mut self, recipient: Address, amount: U256) {
        self.erc20.transfer(recipient, amount)
    }
}

pub mod events {
    use odra::{
        types::{Address, U256},
        Event
    };

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

#[cfg(all(test, feature = "mock-vm"))]
mod tests {

    use odra::{
        assert_events, test_env,
        types::{Address, Balance, OdraError, VmError, U256}
    };

    use crate::{
        erc20::events::Transfer,
        wrapped_native::{
            events::{Deposit, Withdrawal},
            WrappedNativeTokenDeployer, WrappedNativeTokenRef
        }
    };

    fn setup() -> (WrappedNativeTokenRef, Address, Balance, Address, Balance) {
        let token: WrappedNativeTokenRef = WrappedNativeTokenDeployer::init();
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
        // When deploy a contract.
        let (token, _, _, _, _) = setup();

        // Then the contract has the default metadata.
        let metadata = test_env::native_token_metadata();
        assert_eq!(token.total_supply(), U256::zero());
        assert_eq!(token.name(), format!("Wrapped {}", metadata.name));
        assert_eq!(token.symbol(), format!("W{}", metadata.symbol));
        assert_eq!(token.decimals(), metadata.decimals);
    }

    #[test]
    fn test_deposit() {
        // Given a fresh contract.
        let (token, account, account_balance, _, _) = setup();

        // When deposit tokens.
        let deposit_amount = 1_000u32;
        token.with_tokens(deposit_amount).deposit();

        // Then the contract balance is updated.
        assert_eq!(token.balance_of(account), deposit_amount.into());
        // Then the user balance is deducted.
        assert_eq!(
            test_env::token_balance(account),
            account_balance - deposit_amount
        );
        // The events were emitted.
        assert_events!(
            token,
            Transfer {
                from: None,
                to: Some(account),
                amount: deposit_amount.into()
            },
            Deposit {
                account,
                value: deposit_amount.into()
            }
        );
    }

    #[test]
    fn test_minting() {
        // Given a fresh contract.
        let (token, account_1, _, account_2, _) = setup();

        // When two users deposit some tokens.
        let deposit_amount = 1_000u32;

        test_env::set_caller(account_1);
        token.with_tokens(deposit_amount).deposit();

        test_env::set_caller(account_2);
        token.with_tokens(deposit_amount).deposit();

        // Then the total supply in the sum of deposits.
        assert_eq!(
            token.total_supply(),
            (deposit_amount + deposit_amount).into()
        );
        // Then events were emitted.
        assert_events!(token, Transfer, Deposit, Transfer, Deposit);
    }

    #[test]
    fn test_deposit_amount_exceeding_account_balance() {
        test_env::assert_exception(OdraError::VmError(VmError::BalanceExceeded), || {
            // Given a new contract.
            let (token, _, balance, _, _) = setup();
            // When the deposit amount exceeds the user's balance
            // Then an error occurs.
            token.with_tokens(balance + Balance::one()).deposit();
        });
    }

    #[test]
    fn test_withdrawal() {
        // Deposit all tokens in the contract
        let (mut token, account, balance, _, _) = setup();
        token.with_tokens(balance).deposit();

        // When withdraw some tokens
        let withdrawal_amount: U256 = 1_000.into();
        token.withdraw(withdrawal_amount);

        // Then the user has the withdrawn tokens back.
        assert_eq!(test_env::token_balance(account), withdrawal_amount);
        // Then the balance in the contract is deducted.
        assert_eq!(token.balance_of(account), balance - withdrawal_amount);
        // Then events were emitted.
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
        test_env::assert_exception(crate::erc20::errors::Error::InsufficientBalance, || {
            // Given a new contract.
            let (mut token, _, _, _, _) = setup();
            // When the user withdraws amount exceeds the user's deposit
            // Then an error occurs.
            token.withdraw(U256::one());
        });
    }
}
