//! CEP-18 Casper Fungible Token standard implementation.
use crate::cep18::errors::errors::Error;
use crate::cep18::errors::errors::Error::{
    CannotTargetSelfUser, InvalidBurnTarget, InvalidState, MintBurnDisabled, Overflow
};
use crate::cep18::events::{
    Burn, DecreaseAllowance, IncreaseAllowance, Mint, SetAllowance, Transfer
};
use crate::cep18::utils::SecurityBadge;
use core::ptr::read;
use odra::casper_types::VersionCheckResult::Invalid;
use odra::prelude::*;
use odra::{casper_types::U256, Address, Mapping, UnwrapOrRevert, Var};

/// ERC20 token module
#[odra::module]
pub struct Cep18 {
    decimals: Var<u8>,
    symbol: Var<String>,
    name: Var<String>,
    total_supply: Var<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<(Address, Address), U256>,
    security_badges: Mapping<Address, SecurityBadge>,
    modality: Var<u8>
}

#[odra::module]
impl Cep18 {
    /// Initializes the contract with the given metadata, initial supply, security and modality.
    pub fn init(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256,
        minter_list: Vec<Address>,
        admin_list: Vec<Address>,
        modality: Option<u8>
    ) {
        let caller = self.env().caller();
        // set the metadata
        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);

        // mint the initial supply for the caller
        self.balances.set(&caller, initial_supply);
        self.env().emit_event(Mint {
            recipient: caller,
            amount: initial_supply
        });

        // set the security badges
        self.security_badges.set(&caller, SecurityBadge::Admin);

        for admin in admin_list {
            self.security_badges.set(&admin, SecurityBadge::Admin);
        }

        for minter in minter_list {
            self.security_badges.set(&minter, SecurityBadge::Minter);
        }

        // set the modality
        if let Some(modality) = modality {
            self.modality.set(modality);
        }
    }

    /// Returns the name of the token.
    pub fn name(&self) -> String {
        self.name.get_or_revert_with(InvalidState)
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> String {
        self.symbol.get_or_revert_with(InvalidState)
    }

    /// Returns the number of decimals the token uses.
    pub fn decimals(&self) -> u8 {
        self.decimals.get_or_revert_with(InvalidState)
    }

    /// Returns the total supply of the token.
    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    /// Returns the balance of the given address.
    pub fn balance_of(&self, address: &Address) -> U256 {
        self.balances.get_or_default(address)
    }

    /// Returns the amount of tokens the owner has allowed the spender to spend.
    pub fn allowance(&self, owner: &Address, spender: &Address) -> U256 {
        self.allowances.get_or_default(&(*owner, *spender))
    }

    /// Approves the spender to spend the given amount of tokens on behalf of the caller.
    pub fn approve(&mut self, spender: &Address, amount: &U256) {
        let owner = self.env().caller();
        if owner == *spender {
            self.env().revert(CannotTargetSelfUser);
        }

        self.allowances.set(&(owner, *spender), *amount);
        self.env().emit_event(SetAllowance {
            owner,
            spender: *spender,
            allowance: *amount
        });
    }

    /// Decreases the allowance of the spender by the given amount.
    pub fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256) {
        let owner = self.env().caller();
        let allowance = self.allowance(&owner, spender);
        self.allowances
            .set(&(owner, *spender), allowance.saturating_sub(*decr_by));
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
            self.env().revert(CannotTargetSelfUser);
        }
        let allowance = self.allowances.get_or_default(&(owner, *spender));

        self.allowances
            .set(&(owner, *spender), allowance.saturating_add(*inc_by));
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
            self.env().revert(CannotTargetSelfUser);
        }
        self.raw_transfer(&caller, recipient, amount);
    }

    /// Transfers tokens from the owner to the recipient using the spender's allowance.
    pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256) {
        let spender = self.env().caller();

        if *owner == *recipient {
            self.env().revert(CannotTargetSelfUser);
        }

        if amount.is_zero() {
            return;
        }

        let allowance = self.allowance(owner, &spender);

        self.allowances.set(
            &(*owner, *recipient),
            allowance
                .checked_sub(*amount)
                .unwrap_or_revert_with(&self.env(), Error::InsufficientAllowance)
        );
        self.env().emit_event(DecreaseAllowance {
            owner: *owner,
            spender,
            allowance,
            decr_by: *amount
        });

        self.raw_transfer(owner, recipient, amount);
    }

    /// Mints new tokens and assigns them to the given address.
    pub fn mint(&mut self, owner: &Address, amount: &U256) {
        // check if mint_burn is enabled
        if self.modality.get_or_default() == 0 {
            self.env().revert(MintBurnDisabled);
        }

        self.total_supply.add(*amount);
        self.balances.add(owner, *amount);

        self.env().emit_event(Mint {
            recipient: *owner,
            amount: *amount
        });
    }

    /// Burns the given amount of tokens from the given address.
    pub fn burn(&mut self, owner: &Address, amount: &U256) {
        // check if mint_burn is enabled
        if self.modality.get_or_default() == 0 {
            self.env().revert(MintBurnDisabled);
        }

        if self.env().caller() != *owner {
            self.env().revert(InvalidBurnTarget);
        }

        if self.balance_of(owner) < *amount {
            self.env().revert(Error::InsufficientBalance);
        }
        let total_supply = self.total_supply.get_or_default();
        let balance = self.balance_of(owner);

        self.total_supply.set(
            total_supply
                .checked_sub(*amount)
                .unwrap_or_revert_with(&self.env(), Overflow)
        );
        self.balances.set(
            owner,
            balance
                .checked_sub(*amount)
                .unwrap_or_revert_with(&self.env(), Overflow)
        );

        self.env().emit_event(Burn {
            owner: *owner,
            amount: *amount
        });
    }
}

impl Cep18 {
    fn raw_transfer(&mut self, sender: &Address, recipient: &Address, amount: &U256) {
        if *amount > self.balances.get_or_default(sender) {
            self.env().revert(Error::InsufficientBalance)
        }

        self.balances.subtract(sender, *amount);
        self.balances.add(recipient, *amount);

        self.env().emit_event(Transfer {
            sender: *sender,
            recipient: *recipient,
            amount: *amount
        });
    }
}
