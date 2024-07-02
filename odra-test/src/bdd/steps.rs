use odra_core::{
    args::EntrypointArgument,
    casper_event_standard::EventInstance,
    casper_types::{bytesrepr::FromBytes, runtime_args, CLTyped, RuntimeArgs, U256, U512},
    host::{HostEnv, HostRef},
    Address, CallDef, ContractCallResult, EventError, OdraResult
};

pub trait Cep18Token {
    fn change_security(
        &mut self,
        admin_list: Vec<Address>,
        minter_list: Vec<Address>,
        none_list: Vec<Address>
    );
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> u8;
    fn total_supply(&self) -> U256;
    fn balance_of(&self, address: &Address) -> U256;
    fn allowance(&self, owner: &Address, spender: &Address) -> U256;
    fn approve(&mut self, spender: &Address, amount: &U256);
    fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256);
    fn increase_allowance(&mut self, spender: &Address, inc_by: &U256);
    fn transfer(&mut self, recipient: &Address, amount: &U256);
    fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
    fn mint(&mut self, owner: &Address, amount: &U256);
    fn burn(&mut self, owner: &Address, amount: &U256);
}

pub struct Cep18TokenHostRef {
    address: Address,
    env: HostEnv,
    attached_value: U512
}

impl HostRef for Cep18TokenHostRef {
    fn new(address: Address, env: HostEnv) -> Self {
        Self {
            address,
            env,
            attached_value: Default::default()
        }
    }
    fn with_tokens(&self, tokens: U512) -> Self {
        Self {
            address: self.address,
            env: self.env.clone(),
            attached_value: tokens
        }
    }
    fn address(&self) -> &Address {
        &self.address
    }
    fn env(&self) -> &HostEnv {
        &self.env
    }
    fn get_event<T>(&self, index: i32) -> Result<T, EventError>
    where
        T: FromBytes + EventInstance
    {
        self.env.get_event(self, index)
    }
    fn last_call(&self) -> ContractCallResult {
        self.env.last_call_result(self.address)
    }
}

impl Cep18Token for Cep18TokenHostRef {
    fn change_security(
        &mut self,
        admin_list: Vec<Address>,
        minter_list: Vec<Address>,
        none_list: Vec<Address>
    ) {
        self.try_change_security(admin_list, minter_list, none_list)
            .unwrap()
    }

    fn name(&self) -> String {
        self.try_name().unwrap()
    }

    fn symbol(&self) -> String {
        self.try_symbol().unwrap()
    }

    fn decimals(&self) -> u8 {
        self.try_decimals().unwrap()
    }

    fn total_supply(&self) -> U256 {
        self.try_total_supply().unwrap()
    }

    fn balance_of(&self, address: &Address) -> U256 {
        self.try_balance_of(address).unwrap()
    }

    fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.try_allowance(owner, spender).unwrap()
    }

    fn approve(&mut self, spender: &Address, amount: &U256) {
        self.try_approve(spender, amount).unwrap()
    }

    fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256) {
        self.try_decrease_allowance(spender, decr_by).unwrap()
    }

    fn increase_allowance(&mut self, spender: &Address, inc_by: &U256) {
        self.try_increase_allowance(spender, inc_by).unwrap()
    }

    fn transfer(&mut self, recipient: &Address, amount: &U256) {
        self.try_transfer(recipient, amount).unwrap()
    }

    fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        self.try_transfer_from(owner, recipient, amount).unwrap()
    }

    fn mint(&mut self, owner: &Address, amount: &U256) {
        self.try_mint(owner, amount).unwrap()
    }

    fn burn(&mut self, owner: &Address, amount: &U256) {
        self.try_burn(owner, amount).unwrap()
    }
}

impl Cep18TokenHostRef {
    pub fn try_call<T: FromBytes + CLTyped>(&self, name: &str, args: RuntimeArgs) -> OdraResult<T> {
        self.env
            .call_contract(self.address, CallDef::new(name, false, args))
    }

    pub fn try_call_mut<T: FromBytes + CLTyped>(
        &self,
        name: &str,
        args: RuntimeArgs
    ) -> OdraResult<T> {
        self.env
            .call_contract(self.address, CallDef::new(name, true, args))
    }
}

impl Cep18TokenHostRef {
    pub fn try_change_security(
        &mut self,
        admin_list: Vec<Address>,
        minter_list: Vec<Address>,
        none_list: Vec<Address>
    ) -> OdraResult<()> {
        let args = runtime_args! {
            "admin_list" => admin_list,
            "minter_list" => minter_list,
            "none_list" => none_list
        };

        self.try_call("change_security", args)
    }

    pub fn try_name(&self) -> OdraResult<String> {
        self.try_call("name", RuntimeArgs::new())
    }

    pub fn try_symbol(&self) -> OdraResult<String> {
        self.try_call("symbol", RuntimeArgs::new())
    }

    pub fn try_decimals(&self) -> OdraResult<u8> {
        self.try_call("decimals", RuntimeArgs::new())
    }

    pub fn try_total_supply(&self) -> OdraResult<U256> {
        self.try_call("total_supply", RuntimeArgs::new())
    }

    pub fn try_balance_of(&self, address: &Address) -> OdraResult<U256> {
        let args = runtime_args! {
            "address" => address
        };

        self.try_call("balance_of", args)
    }

    pub fn try_allowance(&self, owner: &Address, spender: &Address) -> OdraResult<U256> {
        let args = runtime_args! {
            "owner" => owner,
            "spender" => spender
        };

        self.try_call("allowance", args)
    }

    pub fn try_approve(&mut self, spender: &Address, amount: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => spender,
            "amount" => amount
        };

        self.try_call("approve", args)
    }

    pub fn try_decrease_allowance(&mut self, spender: &Address, decr_by: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => spender,
            "decr_by" => decr_by
        };

        self.try_call_mut("decrease_allowance", args)
    }

    pub fn try_increase_allowance(&mut self, spender: &Address, inc_by: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "spender" => spender,
            "inc_by" => inc_by
        };

        self.try_call_mut("increase_allowance", args)
    }

    pub fn try_transfer(&mut self, recipient: &Address, amount: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "recipient" => recipient,
            "amount" => amount
        };

        self.try_call_mut("transfer", args)
    }

    pub fn try_transfer_from(
        &mut self,
        owner: &Address,
        recipient: &Address,
        amount: &U256
    ) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => owner,
            "recipient" => recipient,
            "amount" => amount
        };

        self.try_call_mut("transfer_from", args)
    }

    pub fn try_mint(&mut self, owner: &Address, amount: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => owner,
            "amount" => amount
        };

        self.try_call_mut("mint", args)
    }

    pub fn try_burn(&mut self, owner: &Address, amount: &U256) -> OdraResult<()> {
        let args = runtime_args! {
            "owner" => owner,
            "amount" => amount
        };

        self.try_call_mut("burn", args)
    }
}
