use odra_env::ContractEnv;
use odra_types::{Address, U256, event::Event};

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
        erc20::ext::mint(&self.erc20, caller, amount);

        ContractEnv::deposit_native_token(caller, amount);
        
        Deposit { account: caller, value: amount }.emit();
    }

    //payable
    pub fn withdraw(&self, amount: U256) {
        let caller = ContractEnv::caller();
        erc20::ext::burn(&self.erc20, caller, amount);

        ContractEnv::withdraw_native_token(caller, amount);

        Withdrawal { account: caller, value: amount }.emit()
    }

    pub fn allowance(&self,owner:Address,spender:Address) ->U256 {
        self.erc20.allowance(owner, spender)
    }

    pub fn balance_of(&self,address:Address) ->U256 {
        self.erc20.balance_of(address)
    }

    pub fn total_supply(&self) ->U256 {
        self.erc20.total_supply()
    }

    pub fn decimals(&self) ->u8 {
        self.erc20.decimals()
    }

    pub fn symbol(&self) ->String {
        self.erc20.symbol()
    }

    pub fn name(&self) ->String {
        self.erc20.name()
    }

    pub fn approve(&self,spender:Address,amount:U256) {
        self.erc20.approve(spender, amount)
    }

    pub fn transfer_from(&self,owner:Address,recipient:Address,amount:U256) {
        self.erc20.transfer_from(owner, recipient, amount)
    }

    pub fn transfer(&self,recipient:Address,amount:U256) {
        self.erc20.transfer(recipient, amount)
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
