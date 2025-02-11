use crate::types::account::Account;
use crate::types::cspr::CSPRAmount;
use crate::virtual_balances::VirtualBalances;
use odra::host::HostEnv;
use odra::prelude::Address;

pub struct BDDEnv {
    env: HostEnv,
    virtual_balances: VirtualBalances
}

impl BDDEnv {
    pub fn new(env: HostEnv) -> Self {
        Self {
            virtual_balances: VirtualBalances::new(env.clone()),
            env
        }
    }

    pub fn get_address(&self, account: &Account) -> Address {
        self.env.get_account(account.account_id())
    }

    pub fn set_caller(&mut self, account: &Account) {
        self.env.set_caller(self.get_address(account));
    }

    pub fn set_balance(&mut self, account: &Account, balance: CSPRAmount) {
        let address = self.get_address(account);
        self.virtual_balances.set_cspr_balance(&address, balance);
    }

    pub fn get_balance(&self, account: &Account) -> CSPRAmount {
        let address = self.get_address(account);
        self.virtual_balances.get_cspr_balance(&address)
    }

    pub fn env(&self) -> &HostEnv {
        &self.env
    }

    pub fn advance_with_rewards(&mut self) {
        self.env.advance_with_auctions(self.env.auction_delay());
    }
}
