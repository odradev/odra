use odra::{
    contract_env, execution_error,
    types::{event::OdraEvent, Address, Balance, BlockTime},
    Mapping, Variable
};

#[odra::module]
pub struct TimeLockWallet {
    balances: Mapping<Address, Balance>,
    lock_expiration_map: Mapping<Address, BlockTime>,
    lock_duration: Variable<BlockTime>
}

#[odra::module]
impl TimeLockWallet {
    #[odra(init)]
    pub fn init(&mut self, lock_duration: BlockTime) {
        self.lock_duration.set(lock_duration);
    }

    #[odra(payable)]
    pub fn deposit(&mut self) {
        // Extract values
        let caller: Address = contract_env::caller();
        let amount: Balance = contract_env::attached_value();
        let current_block_time: BlockTime = contract_env::get_block_time();

        // Multiple lock check
        if self.balances.get(&caller).is_some() {
            contract_env::revert(Error::CannotLockTwice)
        }

        // Update state, emit event
        self.balances.set(&caller, amount);
        self.lock_expiration_map
            .set(&caller, current_block_time + self.lock_duration());
        Deposit {
            address: caller,
            amount
        }
        .emit();
    }

    pub fn withdraw(&mut self, amount: Balance) {
        // Extract values
        let caller: Address = contract_env::caller();
        let current_block_time: BlockTime = contract_env::get_block_time();
        let balance: Balance = self.balances.get_or_default(&caller);

        // Balance check
        if amount > balance {
            contract_env::revert(Error::InsufficientBalance)
        }

        // Lock check
        let lock_expiration_time = self.lock_expiration_map.get_or_default(&caller);
        if current_block_time < lock_expiration_time {
            contract_env::revert(Error::LockIsNotOver)
        }

        // Transfer tokens, emit event
        contract_env::transfer_tokens(caller, amount);
        self.balances.subtract(&caller, amount);
        Withdrawal {
            address: caller,
            amount
        }
        .emit();
    }

    pub fn get_balance(&self, address: Address) -> Balance {
        self.balances.get_or_default(&address)
    }

    pub fn lock_duration(&self) -> BlockTime {
        self.lock_duration.get_or_default()
    }
}

execution_error! {
    pub enum Error {
        LockIsNotOver => 1,
        CannotLockTwice => 2,
        InsufficientBalance => 3
    }
}

#[derive(odra::Event, PartialEq, Eq, Debug)]

pub struct Deposit {
    pub address: Address,
    pub amount: Balance
}

#[derive(odra::Event, PartialEq, Eq, Debug)]
pub struct Withdrawal {
    pub address: Address,
    pub amount: Balance
}

#[cfg(test)]
mod test {
    use odra::{
        assert_events, test_env,
        types::{Address, Balance}
    };

    use crate::tlw::{Deposit, Withdrawal};

    use super::{Error, TimeLockWalletDeployer, TimeLockWalletRef};

    const ONE_DAY_IN_SECONDS: u64 = 60 * 60 * 24;

    fn setup() -> (TimeLockWalletRef, Address, Address) {
        (
            TimeLockWalletDeployer::init(ONE_DAY_IN_SECONDS),
            test_env::get_account(0),
            test_env::get_account(1)
        )
    }

    #[test]
    fn test_deposit() {
        // Given a new contract.
        let (contract, user_1, user_2) = setup();

        // Helper method for a single deposit.
        let single_deposit = |account: Address, deposit: Balance| {
            let balance = test_env::token_balance(account);
            test_env::set_caller(account);
            contract.with_tokens(deposit).deposit();
            let gas_used = test_env::last_call_contract_gas_used();
            let balance_after = test_env::token_balance(account);
            assert_eq!(balance_after + gas_used + deposit, balance);
        };

        // When two users deposit some tokens.
        let user_1_deposit: Balance = 100.into();
        single_deposit(user_1, user_1_deposit);

        let user_2_deposit: Balance = 200.into();
        single_deposit(user_2, user_2_deposit);

        // Then the users' balance is the contract is equal to the deposited amount.
        assert_eq!(contract.get_balance(user_1), user_1_deposit);
        assert_eq!(contract.get_balance(user_2), user_2_deposit);

        // Then two deposit event were emitted.
        assert_events!(
            contract,
            Deposit {
                address: user_1,
                amount: user_1_deposit
            },
            Deposit {
                address: user_2,
                amount: user_2_deposit
            }
        );
    }

    #[test]
    fn second_deposit_for_the_same_user_should_fail() {
        test_env::assert_exception(Error::CannotLockTwice, || {
            // Given a new contract.
            let (contract, _, _) = setup();

            // The user makes the first deposit.
            let deposit: Balance = 100.into();
            contract.with_tokens(deposit).deposit();

            // When the user tries to deposit tokens for the second time, an error occurs.
            contract.with_tokens(deposit).deposit();
        });
    }

    #[test]
    fn test_successful_withdrawal() {
        // Given a contract with the user's deposit.
        let (mut contract, user, _) = setup();
        let deposit_amount: Balance = 100.into();
        contract.with_tokens(deposit_amount).deposit();

        // When the user makes two token withdrawals after the lock is expired.
        test_env::advance_block_time_by(ONE_DAY_IN_SECONDS + 1);
        let balance_before_withdrawals = test_env::token_balance(user);
        let first_withdrawal_amount: Balance = 50.into();
        let second_withdrawal_amount: Balance = 40.into();
        contract.withdraw(first_withdrawal_amount);
        let mut gas_used = test_env::last_call_contract_gas_used();
        contract.withdraw(second_withdrawal_amount);
        gas_used = gas_used + test_env::last_call_contract_gas_used();

        // Then the native token balance is updated.
        assert_eq!(
            test_env::token_balance(user),
            balance_before_withdrawals - gas_used
                + first_withdrawal_amount
                + second_withdrawal_amount
        );

        // Then the user balance in the contract is deducted.
        assert_eq!(
            contract.get_balance(user),
            deposit_amount - first_withdrawal_amount - second_withdrawal_amount
        );

        // Then two Withdrawal events were emitted.
        assert_events!(
            contract,
            Withdrawal {
                address: user,
                amount: first_withdrawal_amount
            },
            Withdrawal {
                address: user,
                amount: second_withdrawal_amount
            }
        );
    }

    #[test]
    fn test_too_early_withdrawal() {
        test_env::assert_exception(Error::LockIsNotOver, || {
            // Given a contract with the user's deposit.
            let (mut contract, _, _) = setup();
            contract.with_tokens(100).deposit();

            // When the user withdraws tokens before the lock is released, an error occurs.
            contract.withdraw(100.into());
        });
    }

    #[test]
    fn test_withdraw_too_much() {
        test_env::assert_exception(Error::InsufficientBalance, || {
            // Given a contract with the user's deposit.
            let (mut contract, _, _) = setup();
            let deposit = 100;
            contract.with_tokens(deposit).deposit();

            // When the user withdraws more tokens than has in the deposit, an error occurs.
            test_env::advance_block_time_by(ONE_DAY_IN_SECONDS + 1);
            let withdrawal = deposit + 1;
            contract.withdraw(withdrawal.into());
        });
    }
}
