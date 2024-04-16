//! CEP-18 Casper Fungible Token standard implementation.
use odra::{Address, casper_types::U256, Mapping, UnwrapOrRevert, Var};
use odra::prelude::*;

use crate::cep18::errors::Error;
use crate::cep18::errors::Error::{
    CannotTargetSelfUser, InsufficientRights, InvalidBurnTarget, InvalidState, MintBurnDisabled,
    Overflow
};
use crate::cep18::events::{
    Burn, DecreaseAllowance, IncreaseAllowance, Mint, SetAllowance, Transfer
};
use crate::cep18::utils::{Cep18Modality, SecurityBadge};

/// CEP-18 token module
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
    #[allow(clippy::too_many_arguments)]
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

    /// Admin EntryPoint to manipulate the security access granted to users.
    /// One user can only possess one access group badge.
    /// Change strength: None > Admin > Minter
    /// Change strength meaning by example: If user is added to both Minter and Admin they will be an
    /// Admin, also if a user is added to Admin and None then they will be removed from having rights.
    /// Beware: do not remove the last Admin because that will lock out all admin functionality.
    pub fn change_security(
        &mut self,
        admin_list: Vec<Address>,
        minter_list: Vec<Address>,
        none_list: Vec<Address>
    ) {
        // if mint burn is disabled, revert
        if self.modality.get_or_default()
            != <Cep18Modality as Into<u8>>::into(Cep18Modality::MintAndBurn)
        {
            self.env().revert(MintBurnDisabled);
        }

        // check if the caller has the admin badge
        let caller = self.env().caller();
        let caller_badge = self
            .security_badges
            .get(&caller)
            .unwrap_or_revert_with(&self.env(), InsufficientRights);

        if !caller_badge.can_admin() {
            self.env().revert(InsufficientRights);
        }

        // set the security badges
        for admin in admin_list {
            self.security_badges.set(&admin, SecurityBadge::Admin);
        }

        for minter in minter_list {
            self.security_badges.set(&minter, SecurityBadge::Minter);
        }

        for none in none_list {
            self.security_badges.set(&none, SecurityBadge::None);
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

        // check if the caller has the minter badge
        let security_badge = self
            .security_badges
            .get(&self.env().caller())
            .unwrap_or_revert_with(&self.env(), InsufficientRights);
        if !security_badge.can_mint() {
            self.env().revert(InsufficientRights);
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

#[cfg(test)]
pub(crate) mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use odra::Address;
    use odra::casper_types::account::AccountHash;
    use odra::casper_types::ContractPackageHash;
    use odra::host::{Deployer, HostEnv, HostRef};

    use crate::cep18_token::{Cep18HostRef, Cep18InitArgs};

    pub const TOKEN_NAME: &str = "Plascoin";
    pub const TOKEN_SYMBOL: &str = "PLS";
    pub const TOKEN_DECIMALS: u8 = 100;
    pub const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;
    pub const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
    pub const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;
    pub const TRANSFER_AMOUNT_1: u64 = 200_001;
    pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
    pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

    pub fn setup(enable_mint_and_burn: bool) -> Cep18HostRef {
        let env = odra_test::env();
        let modality = if enable_mint_and_burn { Some(1) } else { None };
        let init_args = Cep18InitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            minter_list: vec![],
            admin_list: vec![],
            modality
        };
        setup_with_args(&env, init_args)
    }

    pub fn setup_with_args(env: &HostEnv, args: Cep18InitArgs) -> Cep18HostRef {
        Cep18HostRef::deploy(env, args)
    }

    pub fn invert_address(address: Address) -> Address {
        match address {
            Address::Account(hash) => Address::Contract(ContractPackageHash::new(hash.value())),
            Address::Contract(hash) => Address::Account(AccountHash(hash.value()))
        }
    }

    #[test]
    fn should_have_queryable_properties() {
        let cep18_token = setup(false);

        assert_eq!(cep18_token.name(), TOKEN_NAME);
        assert_eq!(cep18_token.symbol(), TOKEN_SYMBOL);
        assert_eq!(cep18_token.decimals(), TOKEN_DECIMALS);
        assert_eq!(cep18_token.total_supply(), TOKEN_TOTAL_SUPPLY.into());

        let owner_key = cep18_token.env().caller();
        let owner_balance = cep18_token.balance_of(&owner_key);
        assert_eq!(owner_balance, TOKEN_TOTAL_SUPPLY.into());

        let contract_balance = cep18_token.balance_of(cep18_token.address());
        assert_eq!(contract_balance, 0.into());

        // Ensures that Account and Contract ownership is respected and we're not keying ownership under
        // the raw bytes regardless of variant.
        let inverted_owner_key = invert_address(owner_key);
        let inverted_owner_balance = cep18_token.balance_of(&inverted_owner_key);
        assert_eq!(inverted_owner_balance, 0.into());
    }
}
