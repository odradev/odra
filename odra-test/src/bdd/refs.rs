#![allow(missing_docs)]

use super::{
    param::{Account, Amount},
    OdraWorld
};
use odra_core::{
    casper_types::{bytesrepr::FromBytes, runtime_args, CLTyped, RuntimeArgs, U256},
    Address, CallDef, OdraResult
};

pub struct Cep18TokenHostRef<'a> {
    address: Address,
    odra_world: &'a mut OdraWorld
}

impl Cep18TokenHostRef<'_> {
    pub fn new<'a>(address: Address, odra_world: &'a mut OdraWorld) -> Cep18TokenHostRef<'a> {
        Cep18TokenHostRef {
            address,
            odra_world
        }
    }

    pub fn try_call<T: FromBytes + CLTyped>(
        &mut self,
        name: &str,
        args: RuntimeArgs
    ) -> OdraResult<T> {
        self.odra_world
            .env()
            .call_contract(self.address, CallDef::new(name, false, args))
    }

    pub fn try_call_mut<T: FromBytes + CLTyped>(
        &mut self,
        name: &str,
        args: RuntimeArgs
    ) -> OdraResult<T> {
        self.odra_world
            .env()
            .call_contract(self.address, CallDef::new(name, true, args))
    }
}

impl<'a> Cep18TokenHostRef<'a> {
    pub fn change_security(
        &mut self,
        admin_list: Vec<Account>,
        minter_list: Vec<Account>,
        none_list: Vec<Account>
    ) {
        self.try_change_security(admin_list, minter_list, none_list)
            .unwrap()
    }

    pub fn name(&mut self) -> String {
        self.try_name().unwrap()
    }

    pub fn symbol(&mut self) -> String {
        self.try_symbol().unwrap()
    }

    pub fn decimals(&mut self) -> u8 {
        self.try_decimals().unwrap()
    }

    pub fn total_supply(&mut self) -> U256 {
        self.try_total_supply().unwrap()
    }

    pub fn balance_of(&mut self, address: &Account) -> U256 {
        self.try_balance_of(address).unwrap()
    }

    pub fn allowance(&mut self, owner: &Account, spender: &Account) -> U256 {
        self.try_allowance(owner, spender).unwrap()
    }

    pub fn approve(&mut self, spender: &Account, amount: &Amount) {
        self.try_approve(spender, amount).unwrap()
    }

    pub fn decrease_allowance(&mut self, spender: &Account, decr_by: &Amount) {
        self.try_decrease_allowance(spender, decr_by).unwrap()
    }

    pub fn increase_allowance(&mut self, spender: &Account, inc_by: &Amount) {
        self.try_increase_allowance(spender, inc_by).unwrap()
    }

    pub fn transfer(&mut self, recipient: &Account, amount: &Amount) {
        self.try_transfer(recipient, amount).unwrap()
    }

    pub fn transfer_from(&mut self, owner: &Account, recipient: &Account, amount: &Amount) {
        self.try_transfer_from(owner, recipient, amount).unwrap()
    }

    pub fn mint(&mut self, owner: &Account, amount: &Amount) {
        self.try_mint(owner, amount).unwrap()
    }

    pub fn burn(&mut self, owner: &Account, amount: &Amount) {
        self.try_burn(owner, amount).unwrap()
    }
}

impl<'a> Cep18TokenHostRef<'a> {
    pub fn try_change_security(
        &mut self,
        admin_list: Vec<Account>,
        minter_list: Vec<Account>,
        none_list: Vec<Account>
    ) -> OdraResult<()> {
        let args = runtime_args! {
            "admin_list" => admin_list.iter().map(|a| self.odra_world.get_address(a.clone()).clone()).collect::<Vec<_>>(),
            "minter_list" => minter_list.iter().map(|a| self.odra_world.get_address(a.clone()).clone()).collect::<Vec<_>>(),
            "none_list" => none_list.iter().map(|a| self.odra_world.get_address(a.clone()).clone()).collect::<Vec<_>>()
        };
        self.try_call("change_security", args)
    }

    pub fn try_name(&mut self) -> OdraResult<String> {
        self.try_call("name", RuntimeArgs::new())
    }

    pub fn try_symbol(&mut self) -> OdraResult<String> {
        self.try_call("symbol", RuntimeArgs::new())
    }

    pub fn try_decimals(&mut self) -> OdraResult<u8> {
        self.try_call("decimals", RuntimeArgs::new())
    }

    pub fn try_total_supply(&mut self) -> OdraResult<U256> {
        self.try_call("total_supply", RuntimeArgs::new())
    }

    pub fn try_balance_of(&mut self, address: &Account) -> OdraResult<U256> {
        let args = runtime_args! {
            "address" => self.odra_world.get_address(address.clone()).clone()
        };

        self.try_call("balance_of", args)
    }

    pub fn try_allowance(&mut self, owner: &Account, spender: &Account) -> OdraResult<U256> {
        let args = runtime_args! {
            "owner" => self.odra_world.get_address(owner.clone()).clone(),
            "spender" => self.odra_world.get_address(spender.clone()).clone()
        };

        self.try_call("allowance", args)
    }

    pub fn try_approve(&mut self, spender: &Account, amount: &Amount) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => self.odra_world.get_address(spender.clone()).clone(),
            "amount" => **amount
        };

        self.try_call("approve", args)
    }

    pub fn try_decrease_allowance(
        &mut self,
        spender: &Account,
        decr_by: &Amount
    ) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => self.odra_world.get_address(spender.clone()).clone(),
            "decr_by" => **decr_by
        };

        self.try_call_mut("decrease_allowance", args)
    }

    pub fn try_increase_allowance(&mut self, spender: &Account, inc_by: &Amount) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => self.odra_world.get_address(spender.clone()).clone(),
            "inc_by" => **inc_by
        };

        self.try_call_mut("increase_allowance", args)
    }

    pub fn try_transfer(&mut self, recipient: &Account, amount: &Amount) -> OdraResult<()> {
        let args = runtime_args! {
            "recipient" => self.odra_world.get_address(recipient.clone()).clone(),
            "amount" => **amount
        };

        self.try_call_mut("transfer", args)
    }

    pub fn try_transfer_from(
        &mut self,
        owner: &Account,
        recipient: &Account,
        amount: &Amount
    ) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => self.odra_world.get_address(owner.clone()).clone(),
            "recipient" => self.odra_world.get_address(recipient.clone()).clone(),
            "amount" => **amount
        };

        self.try_call_mut("transfer_from", args)
    }

    pub fn try_mint(&mut self, owner: &Account, amount: &Amount) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => self.odra_world.get_address(owner.clone()).clone(),
            "amount" => **amount
        };

        self.try_call_mut("mint", args)
    }

    pub fn try_burn(&mut self, owner: &Account, amount: &Amount) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => self.odra_world.get_address(owner.clone()).clone(),
            "amount" => **amount
        };

        self.try_call_mut("burn", args)
    }
}
