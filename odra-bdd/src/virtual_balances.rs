use crate::types::cspr::CSPRAmount;
use odra::host::HostEnv;
use odra::prelude::Address;
use std::collections::HashMap;

pub struct VirtualBalances {
    env: HostEnv,
    pub current: HashMap<Address, CSPRAmount>,
    pub initial: HashMap<Address, CSPRAmount>
}

impl VirtualBalances {
    pub fn new(env: HostEnv) -> Self {
        Self {
            env,
            current: HashMap::new(),
            initial: HashMap::new()
        }
    }

    pub fn init(&mut self, address: &Address, amount: CSPRAmount) {
        assert!(
            !self.current.contains_key(address),
            "Cannot set cspr balance twice"
        );

        self.current.insert(*address, amount);

        self.initial
            .insert(*address, CSPRAmount::new(self.env.balance_of(address), 9));
    }

    pub fn get(&self, address: &Address) -> CSPRAmount {
        let current = self.current.get(address).unwrap();
        let balance = current.amount() + self.env.balance_of(address);

        let result = balance
            .checked_sub(self.initial.get(address).unwrap().amount())
            .unwrap();
        CSPRAmount::new(result, 9)
    }

    pub fn set_cspr_balance(&mut self, account: &Address, amount: CSPRAmount) {
        self.init(account, amount);
    }

    // gets relative amount of motes of the account
    pub fn get_cspr_balance(&self, address: &Address) -> CSPRAmount {
        self.get(address)
    }
}
