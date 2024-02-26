//! This is an example of a TimeLockWallet.
use odra::prelude::*;
use odra::{casper_types::U512, Address, Event, Mapping, OdraError, Var};

/// TimeLockWallet contract.
#[odra::module]
pub struct TimeLockWallet {
    balances: Mapping<Address, U512>,
    lock_expiration_map: Mapping<Address, u64>,
    lock_duration: Var<u64>
}

#[odra::module]
impl TimeLockWallet {
    /// Initializes the contract with the lock duration.
    pub fn init(&mut self, lock_duration: u64) {
        self.lock_duration.set(lock_duration);
    }

    /// Deposits the tokens into the contract.
    #[odra(payable)]
    pub fn deposit(&mut self) {
        // Extract values
        let caller: Address = self.env().caller();
        let amount: U512 = self.env().attached_value();
        let current_block_time: u64 = self.env().get_block_time();

        // Multiple lock check
        if self.balances.get(&caller).is_some() {
            self.env().revert(Error::CannotLockTwice)
        }

        // Update state, emit event
        self.balances.set(&caller, amount);
        self.lock_expiration_map
            .set(&caller, current_block_time + self.lock_duration());
        self.env().emit_event(Deposit {
            address: caller,
            amount
        });
    }

    /// Withdraws the tokens from the contract.
    pub fn withdraw(&mut self, amount: &U512) {
        // Extract values
        let caller: Address = self.env().caller();
        let current_block_time: u64 = self.env().get_block_time();
        let balance: U512 = self.balances.get_or_default(&caller);

        // Balance check
        if *amount > balance {
            self.env().revert(Error::InsufficientBalance)
        }

        // Lock check
        let lock_expiration_time = self.lock_expiration_map.get_or_default(&caller);
        if current_block_time < lock_expiration_time {
            self.env().revert(Error::LockIsNotOver)
        }

        // Transfer tokens, emit event
        self.env().transfer_tokens(&caller, amount);
        self.balances.subtract(&caller, *amount);
        self.env().emit_event(Withdrawal {
            address: caller,
            amount: *amount
        });
    }

    /// Returns the balance of the given account.
    pub fn get_balance(&self, address: &Address) -> U512 {
        self.balances.get_or_default(address)
    }

    /// Returns the lock duration.
    pub fn lock_duration(&self) -> u64 {
        self.lock_duration.get_or_default()
    }
}

#[derive(OdraError)]
/// Errors that may occur during the contract execution.
pub enum Error {
    /// Cannot withdraw funds, the lock period is not over.
    LockIsNotOver = 1,
    /// A user deposit funds the second and the next time.
    CannotLockTwice = 2,
    /// A user deposits more funds he/she owns.
    InsufficientBalance = 3
}

/// Deposit event.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct Deposit {
    /// The address of the user who deposited the tokens.
    pub address: Address,
    /// The amount of the deposited tokens.
    pub amount: U512
}

/// Withdrawal event.
#[derive(Event, PartialEq, Eq, Debug)]
pub struct Withdrawal {
    /// The address of the user who withdrew the tokens.
    pub address: Address,
    /// The amount of the withdrawn tokens.
    pub amount: U512
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::contracts::tlw::{Deposit, Withdrawal};
    use odra::{
        host::{Deployer, HostRef},
        Address
    };

    const ONE_DAY_IN_SECONDS: u64 = 60 * 60 * 24;

    fn setup() -> (TimeLockWalletHostRef, Address, Address) {
        let test_env = odra_test::env();
        (
            TimeLockWalletHostRef::deploy(
                &test_env,
                TimeLockWalletInitArgs {
                    lock_duration: ONE_DAY_IN_SECONDS
                }
            ),
            test_env.get_account(0),
            test_env.get_account(1)
        )
    }

    #[test]
    fn test_deposit() {
        // Given a new contract.
        let (contract, user_1, user_2) = setup();
        let test_env = contract.env().clone();

        // Helper method for a single deposit.
        let single_deposit = |account: Address, deposit: U512| {
            let balance = test_env.balance_of(&account);
            test_env.set_caller(account);
            contract.with_tokens(deposit).deposit();
            let balance_after = test_env.balance_of(&account);
            assert_eq!(balance_after + deposit, balance);
        };

        // When two users deposit some tokens.
        let user_1_deposit: U512 = 100.into();
        single_deposit(user_1, user_1_deposit);

        let user_2_deposit: U512 = 200.into();
        single_deposit(user_2, user_2_deposit);

        // Then the users' balance is the contract is equal to the deposited amount.
        assert_eq!(contract.get_balance(&user_1), user_1_deposit);
        assert_eq!(contract.get_balance(&user_2), user_2_deposit);

        // Then two deposit event were emitted.
        test_env.emitted_event(
            contract.address(),
            &Deposit {
                address: user_1,
                amount: user_1_deposit
            }
        );
        test_env.emitted_event(
            contract.address(),
            &Deposit {
                address: user_2,
                amount: user_2_deposit
            }
        );
    }

    #[test]
    fn second_deposit_for_the_same_user_should_fail() {
        // Given a new contract.
        let (contract, _, _) = setup();

        // The user makes the first deposit.
        let deposit: U512 = 100.into();
        contract.with_tokens(deposit).deposit();

        // When the user tries to deposit tokens for the second time, an error occurs.
        assert_eq!(
            contract.with_tokens(deposit).try_deposit().unwrap_err(),
            Error::CannotLockTwice.into()
        );
    }

    #[test]
    fn test_successful_withdrawal() {
        // Given a contract with the user's deposit.
        let (mut contract, user, _) = setup();
        let test_env = contract.env().clone();
        let deposit_amount: U512 = 100.into();
        contract.with_tokens(deposit_amount).deposit();

        // When the user makes two token withdrawals after the lock is expired.
        test_env.advance_block_time(ONE_DAY_IN_SECONDS + 1);
        let balance_before_withdrawals = test_env.balance_of(&user);
        let first_withdrawal_amount: U512 = 50.into();
        let second_withdrawal_amount: U512 = 40.into();
        contract.withdraw(&first_withdrawal_amount);
        contract.withdraw(&second_withdrawal_amount);

        // Then the native token balance is updated.
        assert_eq!(
            test_env.balance_of(&user),
            balance_before_withdrawals + first_withdrawal_amount + second_withdrawal_amount
        );

        // Then the user balance in the contract is deducted.
        assert_eq!(
            contract.get_balance(&user),
            deposit_amount - first_withdrawal_amount - second_withdrawal_amount
        );

        // Then two Withdrawal events were emitted.
        test_env.emitted_event(
            contract.address(),
            &Withdrawal {
                address: user,
                amount: first_withdrawal_amount
            }
        );
        test_env.emitted_event(
            contract.address(),
            &Withdrawal {
                address: user,
                amount: second_withdrawal_amount
            }
        );
    }

    #[test]
    fn test_too_early_withdrawal() {
        // Given a contract with the user's deposit.
        let (mut contract, _, _) = setup();
        contract.with_tokens(100.into()).deposit();

        // When the user withdraws tokens before the lock is released, an error occurs.
        assert_eq!(
            contract.try_withdraw(&100.into()).unwrap_err(),
            Error::LockIsNotOver.into()
        );
    }

    #[test]
    fn test_withdraw_too_much() {
        // Given a contract with the user's deposit.
        let (mut contract, _, _) = setup();
        let deposit = 100;
        contract.with_tokens(deposit.into()).deposit();

        // When the user withdraws more tokens than has in the deposit, an error occurs.
        contract.env().advance_block_time(ONE_DAY_IN_SECONDS + 1);
        let withdrawal = deposit + 1;
        assert_eq!(
            contract.try_withdraw(&withdrawal.into()).unwrap_err(),
            Error::InsufficientBalance.into()
        );
    }
}
