//! CEP-18 Casper Fungible Token standard implementation.
use odra::casper_types::U256;
use odra::prelude::*;

use crate::cep18::errors::Error;

use crate::cep18::events::{
    Burn, DecreaseAllowance, IncreaseAllowance, Mint, SetAllowance, Transfer, TransferFrom
};
use crate::cep18::storage::{
    Cep18AllowancesStorage, Cep18BalancesStorage, Cep18DecimalsStorage, Cep18NameStorage,
    Cep18SymbolStorage, Cep18TotalSupplyStorage
};

/// CEP-18 token module
#[odra::module(
    events = [
        Mint, Burn, SetAllowance, IncreaseAllowance, DecreaseAllowance, Transfer, TransferFrom
    ],
    errors = Error
)]
pub struct Cep18 {
    decimals: SubModule<Cep18DecimalsStorage>,
    symbol: SubModule<Cep18SymbolStorage>,
    name: SubModule<Cep18NameStorage>,
    total_supply: SubModule<Cep18TotalSupplyStorage>,
    balances: SubModule<Cep18BalancesStorage>,
    allowances: SubModule<Cep18AllowancesStorage>
}

#[odra::module]
impl Cep18 {
    /// Initializes the contract with the given metadata, initial supply.
    pub fn init(&mut self, symbol: String, name: String, decimals: u8, initial_supply: U256) {
        let caller = self.env().caller();

        // Set the metadata
        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);

        if !initial_supply.is_zero() {
            // If the initial supply is not zero:
            // - mint the initial supply to the caller,
            // - emit the `Mint` event.
            self.balances.set(&caller, initial_supply);
            self.env().emit_event(Mint {
                recipient: caller,
                amount: initial_supply
            });
        } else {
            // If the initial supply is zero, initialize `balances`.
            self.balances.init();
        }

        // Initialize allowances.
        self.allowances.init();
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        self.name.get()
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        self.symbol.get()
    }

    /// Returns the number of decimals the token uses.
    pub fn decimals(&self) -> u8 {
        self.decimals.get()
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    /// Returns the balance of the given address.
    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get(address).unwrap_or_default()
    }

    /// Returns the amount of tokens the owner has allowed the spender to spend.
    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances.get_or_default(owner, spender)
    }

    /// Approves the spender to spend the given amount of tokens on behalf of the caller.
    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        let owner = self.env().caller();
        if owner == *spender {
            self.env().revert(Error::CannotTargetSelfUser);
        }
        self.raw_approve(&owner, spender, amount);
    }

    /// Decreases the allowance of the spender by the given amount.
    pub fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256) {
        let owner = self.env().caller();
        let allowance = self.allowance(&owner, spender);
        self.allowances
            .set(&owner, spender, allowance.saturating_sub(*decr_by));
        self.env().emit_event(DecreaseAllowance {
            owner,
            spender: *spender,
            allowance,
            decr_by: *decr_by
        });
    }

    /// Increases the allowance of the spender by the given amount.
    pub fn increase_allowance(&mut self, spender: &Address, inc_by: &U256) {
        let owner = self.env().caller();
        if owner == *spender {
            self.env().revert(Error::CannotTargetSelfUser);
        }
        let allowance = self.allowances.get_or_default(&owner, spender);

        self.allowances
            .set(&owner, spender, allowance.saturating_add(*inc_by));
        self.env().emit_event(IncreaseAllowance {
            owner,
            spender: *spender,
            allowance,
            inc_by: *inc_by
        });
    }

    /// Transfers tokens from the caller to the recipient.
    pub fn transfer(&mut self, recipient: &Address, amount: &U256) {
        let caller = self.env().caller();
        if caller == *recipient {
            self.env().revert(Error::CannotTargetSelfUser);
        }

        self.raw_transfer(&caller, recipient, amount);

        self.env().emit_event(Transfer {
            sender: caller,
            recipient: *recipient,
            amount: *amount
        });
    }

    /// Transfers tokens from the owner to the recipient using the spender's allowance.
    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let spender = self.env().caller();

        if owner == recipient {
            self.env().revert(Error::CannotTargetSelfUser);
        }

        if amount.is_zero() {
            return;
        }

        let allowance = self.allowance(owner, &spender);

        self.allowances.set(
            owner,
            recipient,
            allowance
                .checked_sub(*amount)
                .unwrap_or_revert_with(self, Error::InsufficientAllowance)
        );
        self.raw_transfer(owner, recipient, amount);

        self.env().emit_event(TransferFrom {
            spender,
            owner: *owner,
            recipient: *recipient,
            amount: *amount
        });
    }
}

impl Cep18 {
    /// Transfers tokens from the sender to the recipient without checking the permissions.
    pub fn raw_transfer(&mut self, sender: &Address, recipient: &Address, amount: &U256) {
        if amount > &self.balance_of(sender) {
            self.env().revert(Error::InsufficientBalance)
        }

        if amount > &U256::zero() {
            self.balances.subtract(sender, *amount);
            self.balances.add(recipient, *amount);
        }
    }

    /// Mints new tokens and assigns them to the given address without checking the permissions.
    pub fn raw_mint(&mut self, owner: &Address, amount: &U256) {
        self.total_supply.add(*amount);
        self.balances.add(owner, *amount);

        self.env().emit_event(Mint {
            recipient: *owner,
            amount: *amount
        });
    }

    /// Approves the spender to spend the given amount of tokens on behalf of the owner without checking the permissions.
    pub fn raw_approve(&mut self, owner: &Address, spender: &Address, amount: &U256) {
        self.allowances.set(owner, spender, *amount);
        self.env().emit_event(SetAllowance {
            owner: *owner,
            spender: *spender,
            allowance: *amount
        });
    }

    /// Burns the given amount of tokens from the given address without checking the permissions.
    pub fn raw_burn(&mut self, owner: &Address, amount: &U256) {
        if &self.balance_of(owner) < amount {
            self.env().revert(Error::InsufficientBalance);
        }

        self.total_supply.subtract(*amount);
        self.balances.subtract(owner, *amount);

        self.env().emit_event(Burn {
            owner: *owner,
            amount: *amount
        });
    }

    /// Set name of the token.
    pub fn set_name(&mut self, name: String) {
        self.name.set(name);
    }

    /// Set symbol of the token.
    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol.set(symbol);
    }

    /// Set decimals of the token.
    pub fn set_decimals(&mut self, decimals: u8) {
        self.decimals.set(decimals);
    }
}

pub(crate) mod utils {
    #![allow(missing_docs)]
    #![allow(dead_code)]

    use crate::access::Ownable;

    use super::*;

    #[odra::odra_error]
    pub enum Error {
        CantMint = 99,
        CantBurn = 100
    }

    #[odra::module]
    pub struct Cep18Example {
        token: SubModule<Cep18>,
        ownable: SubModule<Ownable>
    }

    #[odra::module]
    impl Cep18Example {
        delegate! {
            to self.token {
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
            }
        }

        pub fn init(&mut self, symbol: String, name: String, decimals: u8, initial_supply: U256) {
            let caller = self.env().caller();
            self.ownable.init(caller);
            self.token.init(symbol, name, decimals, initial_supply);
        }

        pub fn mint(&mut self, owner: &Address, amount: &U256) {
            if self.env().caller() != self.ownable.get_owner() {
                self.env().revert(Error::CantMint);
            }
            self.token.raw_mint(owner, amount);
        }

        pub fn burn(&mut self, owner: &Address, amount: &U256) {
            if self.env().caller() != *owner {
                self.env().revert(Error::CantBurn);
            }
            self.token.raw_burn(owner, amount);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use alloc::string::ToString;

    use odra::casper_types::account::AccountHash;
    use odra::casper_types::contracts::ContractPackageHash;
    use odra::host::{Deployer, HostEnv, HostRef};
    use odra::prelude::*;

    use super::utils::{Cep18Example, Cep18ExampleHostRef, Cep18ExampleInitArgs};

    pub const TOKEN_NAME: &str = "Plascoin";
    pub const TOKEN_SYMBOL: &str = "PLS";
    pub const TOKEN_DECIMALS: u8 = 100;
    pub const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;
    pub const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
    pub const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;
    pub const TRANSFER_AMOUNT_1: u64 = 200_001;
    pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
    pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

    pub fn setup() -> Cep18ExampleHostRef {
        let env = odra_test::env();
        let init_args = Cep18ExampleInitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into()
        };
        setup_with_args(&env, init_args)
    }

    pub fn setup_with_args(env: &HostEnv, args: Cep18ExampleInitArgs) -> Cep18ExampleHostRef {
        Cep18Example::deploy(env, args)
    }

    pub fn invert_address(address: Address) -> Address {
        match address {
            Address::Account(hash) => Address::Contract(ContractPackageHash::new(hash.value())),
            Address::Contract(hash) => Address::Account(AccountHash(hash.value()))
        }
    }

    #[test]
    fn should_have_queryable_properties() {
        let cep18_token = setup();

        assert_eq!(cep18_token.name(), TOKEN_NAME);
        assert_eq!(cep18_token.symbol(), TOKEN_SYMBOL);
        assert_eq!(cep18_token.decimals(), TOKEN_DECIMALS);
        assert_eq!(cep18_token.total_supply(), TOKEN_TOTAL_SUPPLY.into());

        let owner_key = cep18_token.env().caller();
        let owner_balance = cep18_token.balance_of(&owner_key);
        assert_eq!(owner_balance, TOKEN_TOTAL_SUPPLY.into());

        let contract_balance = cep18_token.balance_of(&cep18_token.address());
        assert_eq!(contract_balance, 0.into());

        // Ensures that Account and Contract ownership is respected, and we're not keying ownership under
        // the raw bytes regardless of variant.
        let inverted_owner_key = invert_address(owner_key);
        let inverted_owner_balance = cep18_token.balance_of(&inverted_owner_key);
        assert_eq!(inverted_owner_balance, 0.into());
    }
}
