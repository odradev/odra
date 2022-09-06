use odra_env::ContractEnv;
use odra_types::{U256, event::Event};

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
        self.erc20.init(token.symbol(), token.name(), token.decimals());
    } 

    //payable
    pub fn deposit(&self, amount: U256) {
        let caller = ContractEnv::caller();
        erc20::traits::mint(&self.erc20, caller, amount);

        ContractEnv::deposit_native_token(caller, amount);
        
        Deposit { account: caller, value: amount }.emit();
    }

    //payable
    pub fn withdraw(&self, amount: U256) {
        let caller = ContractEnv::caller();
        erc20::traits::burn(&self.erc20, caller, amount);

        ContractEnv::withdraw_native_token(caller, amount);

        Withdrawal { account: caller, value: amount }.emit()
    }
}

pub mod events {
    use odra_proc_macros::Event;
    use odra_types::{Address, U256};

    #[derive(Event)]
    pub struct Deposit {
        pub account: Address,
        pub value: U256,
    }

    #[derive(Event)]
    pub struct Withdrawal {
        pub account: Address,
        pub value: U256,
    }
}
