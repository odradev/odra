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
    use odra::{assert_events, test_env, types::Balance};

    use crate::tlw::{Deposit, Withdrawal};

    use super::{Error, TimeLockWallet, TimeLockWalletRef};

    const ONE_DAY_IN_SECONDS: u64 = 60 * 60 * 24;

    fn setup() -> TimeLockWalletRef {
        TimeLockWallet::deploy_init(ONE_DAY_IN_SECONDS)
    }

    #[test]
    fn test_deposit() {
        let (user_1, user_2) = (test_env::get_account(0), test_env::get_account(1));
        let contract = setup();

        let user_1_deposit: Balance = 100.into();
        test_env::set_caller(user_1);
        contract.with_tokens(user_1_deposit).deposit();

        let user_2_deposit: Balance = 200.into();
        test_env::set_caller(user_2);
        contract.with_tokens(user_2_deposit).deposit();

        assert_eq!(contract.get_balance(user_1), user_1_deposit);

        assert_eq!(contract.get_balance(user_2), user_2_deposit);

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
            let deposit: Balance = 100.into();
            let contract = setup();

            contract.with_tokens(deposit).deposit();
            contract.with_tokens(deposit).deposit();
        });
    }

    #[test]
    fn test_successful_withdrawal() {
        let user = test_env::get_account(0);
        let deposit_amount: Balance = 100.into();
        let first_withdrawal_amount: Balance = 50.into();
        let second_withdrawal_amount: Balance = 40.into();

        let mut contract = setup();
        contract.with_tokens(deposit_amount).deposit();

        test_env::advance_block_time_by(ONE_DAY_IN_SECONDS + 1);

        contract.withdraw(first_withdrawal_amount);
        contract.withdraw(second_withdrawal_amount);

        assert_eq!(
            contract.get_balance(user),
            deposit_amount - first_withdrawal_amount - second_withdrawal_amount
        );

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
            let mut contract = setup();
            contract.with_tokens(100).deposit();
            contract.withdraw(100.into());
        });
    }

    #[test]
    fn test_withdraw_too_much() {
        let deposit = 100;
        let withdrawal = deposit + 1;

        test_env::assert_exception(Error::InsufficientBalance, || {
            let mut contract = setup();
            contract.with_tokens(deposit).deposit();
            contract.withdraw(withdrawal.into());
        });
    }
}
