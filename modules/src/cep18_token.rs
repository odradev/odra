//! CEP-18 Casper Fungible Token standard implementation.
use odra::prelude::*;
use odra::{casper_types::U256, Address, Mapping, UnwrapOrRevert, Var};

use crate::cep18::errors::errors::Error;
use crate::cep18::errors::errors::Error::{
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
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use core::ops::Add;

    use odra::casper_types::account::AccountHash;
    use odra::casper_types::ContractPackageHash;
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostEnv, HostRef};
    use odra::Address;
    use odra::ExecutionError::AdditionOverflow;

    use crate::cep18::errors::errors::Error::{
        CannotTargetSelfUser, InsufficientAllowance, InsufficientBalance, InsufficientRights,
        MintBurnDisabled
    };
    use crate::cep18::utils::Cep18Modality;
    use crate::cep18_token::{Cep18HostRef, Cep18InitArgs};

    pub const TOKEN_NAME: &str = "Plascoin";
    pub const TOKEN_SYMBOL: &str = "PLS";
    pub const TOKEN_DECIMALS: u8 = 100;
    pub const TOKEN_TOTAL_SUPPLY: u64 = 1_000_000_000;
    pub const TOKEN_OWNER_AMOUNT_1: u64 = 1_000_000;
    pub const TOKEN_OWNER_AMOUNT_2: u64 = 2_000_000;
    pub const TRANSFER_AMOUNT_1: u64 = 200_001;
    pub const TRANSFER_AMOUNT_2: u64 = 19_999;
    pub const ALLOWANCE_AMOUNT_1: u64 = 456_789;
    pub const ALLOWANCE_AMOUNT_2: u64 = 87_654;

    fn setup(enable_mint_and_burn: bool) -> Cep18HostRef {
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

    fn setup_with_args(env: &HostEnv, args: Cep18InitArgs) -> Cep18HostRef {
        Cep18HostRef::deploy(env, args)
    }

    fn invert_address(address: Address) -> Address {
        match address {
            Address::Account(hash) => Address::Contract(ContractPackageHash::new(hash.value())),
            Address::Contract(hash) => Address::Account(AccountHash(hash.value()))
        }
    }

    fn test_approve_for(
        cep18_token: &mut Cep18HostRef,
        sender: Address,
        owner: Address,
        spender: Address
    ) {
        let amount = TRANSFER_AMOUNT_1.into();

        // initial allowance is zero
        assert_eq!(cep18_token.allowance(&owner, &spender), 0.into());

        // when the owner approves the spender to spend tokens on their behalf
        cep18_token.env().set_caller(sender);
        cep18_token.approve(&spender, &amount);

        // then the allowance is set
        assert_eq!(cep18_token.allowance(&owner, &spender), amount);

        // when new allowance is set
        cep18_token.approve(&spender, &(amount.add(U256::one())));

        // then the allowance is updated
        assert_eq!(
            cep18_token.allowance(&owner, &spender),
            amount.add(U256::one())
        );

        // swapping address types
        let inverted_owner = invert_address(owner);
        let inverted_spender = invert_address(spender);
        assert_eq!(
            cep18_token.allowance(&inverted_owner, &inverted_spender),
            U256::zero()
        );
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

    #[test]
    fn test_mint_and_burn() {
        let mut cep18_token = setup(true);

        let alice = cep18_token.env().get_account(1);
        let bob = cep18_token.env().get_account(2);
        let owner = cep18_token.env().caller();
        let initial_supply = cep18_token.total_supply();
        let amount = TRANSFER_AMOUNT_1.into();

        cep18_token.mint(&alice, &amount);
        assert_eq!(cep18_token.total_supply(), initial_supply + amount);

        cep18_token.mint(&bob, &amount);
        assert_eq!(cep18_token.total_supply(), initial_supply + amount + amount);
        assert_eq!(cep18_token.balance_of(&bob), amount);
        assert_eq!(cep18_token.balance_of(&alice), amount);
        assert_eq!(cep18_token.balance_of(&owner), initial_supply);

        cep18_token.burn(&owner, &amount);
        assert_eq!(cep18_token.total_supply(), initial_supply + amount);
        assert_eq!(cep18_token.balance_of(&alice), amount);
        assert_eq!(cep18_token.balance_of(&bob), amount);
        assert_eq!(
            cep18_token.balance_of(&owner),
            initial_supply.saturating_sub(amount)
        );
    }

    #[test]
    fn test_should_not_mint_above_limits() {
        let mut cep18_token = setup(true);
        let mint_amount = U256::MAX;

        let alice = cep18_token.env().get_account(1);
        let bob = cep18_token.env().get_account(2);

        cep18_token.mint(&alice, &U256::from(TOKEN_OWNER_AMOUNT_1));
        cep18_token.mint(&bob, &U256::from(TOKEN_OWNER_AMOUNT_2));

        assert_eq!(
            cep18_token.balance_of(&alice),
            U256::from(TOKEN_OWNER_AMOUNT_1)
        );

        let result = cep18_token.try_mint(&alice, &mint_amount);
        assert_eq!(result.err().unwrap(), AdditionOverflow.into());
    }

    #[test]
    fn should_not_burn_above_balance() {
        let mut cep18_token = setup(true);
        let alice = cep18_token.env().get_account(1);
        let bob = cep18_token.env().get_account(2);

        cep18_token.mint(&alice, &U256::from(TOKEN_OWNER_AMOUNT_1));
        cep18_token.mint(&bob, &U256::from(TOKEN_OWNER_AMOUNT_2));

        assert_eq!(
            cep18_token.balance_of(&alice),
            U256::from(TOKEN_OWNER_AMOUNT_1)
        );

        cep18_token.env().set_caller(alice);
        let result = cep18_token.try_burn(&alice, &U256::from(TOKEN_OWNER_AMOUNT_1 + 1));
        assert_eq!(result.err().unwrap(), InsufficientBalance.into());
    }

    #[test]
    fn should_not_mint_or_burn_when_disabled() {
        let mut cep18_token = setup(false);
        let alice = cep18_token.env().get_account(1);
        let amount = TRANSFER_AMOUNT_1.into();

        let result = cep18_token.try_mint(&alice, &amount);
        assert_eq!(result.err().unwrap(), MintBurnDisabled.into());

        let result = cep18_token.try_burn(&alice, &amount);
        assert_eq!(result.err().unwrap(), MintBurnDisabled.into());
    }

    #[test]
    fn test_security_no_rights() {
        // given a token with mint and burn enabled
        let mut cep18_token = setup(true);
        let alice = cep18_token.env().get_account(1);
        let bob = cep18_token.env().get_account(2);
        let amount = TRANSFER_AMOUNT_1.into();

        // an admin can mint tokens
        cep18_token.mint(&alice, &amount);
        cep18_token.mint(&bob, &amount);

        assert_eq!(cep18_token.balance_of(&alice), amount);
        assert_eq!(cep18_token.balance_of(&bob), amount);

        // user without permissions cannot mint tokens
        cep18_token.env().set_caller(alice);
        let result = cep18_token.try_mint(&bob, &amount);
        assert_eq!(result.err().unwrap(), InsufficientRights.into());

        // but can burn their own tokens
        cep18_token.burn(&alice, &amount);
        assert_eq!(cep18_token.balance_of(&alice), 0.into());
        assert_eq!(cep18_token.balance_of(&bob), amount);
    }

    #[test]
    fn test_security_minter_rights() {
        // given a token with mint and burn enabled, and alice set as minter
        let env = odra_test::env();
        let alice = env.get_account(1);
        let bob = env.get_account(2);
        let args = Cep18InitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            minter_list: vec![alice],
            admin_list: vec![],
            modality: Some(1)
        };
        let mut cep18_token = setup_with_args(&env, args);
        let amount = TRANSFER_AMOUNT_1.into();

        // alice can mint tokens
        cep18_token.env().set_caller(alice);
        cep18_token.mint(&bob, &amount);
        assert_eq!(cep18_token.balance_of(&bob), amount);

        // and bob cannot
        cep18_token.env().set_caller(bob);
        let result = cep18_token.try_mint(&alice, &amount);
        assert_eq!(result.err().unwrap(), InsufficientRights.into());
    }

    #[test]
    fn test_change_security() {
        // given a token with mint and burn enabled, and alice set as an admin
        let env = odra_test::env();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let args = Cep18InitArgs {
            symbol: TOKEN_SYMBOL.to_string(),
            name: TOKEN_NAME.to_string(),
            decimals: TOKEN_DECIMALS,
            initial_supply: TOKEN_TOTAL_SUPPLY.into(),
            minter_list: vec![],
            admin_list: vec![alice],
            modality: Some(Cep18Modality::MintAndBurn.into())
        };
        let mut cep18_token = setup_with_args(&env, args);

        // when alice removes an owner from admin list
        cep18_token.env().set_caller(alice);
        cep18_token.change_security(vec![], vec![], vec![owner]);

        // then the owner cannot mint tokens
        cep18_token.env().set_caller(owner);
        let result = cep18_token.try_mint(&owner, &100.into());
        assert_eq!(result.err().unwrap(), InsufficientRights.into());
    }

    #[test]
    fn should_approve_funds() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let token_address = *cep18_token.address();
        let another_token = setup_with_args(
            cep18_token.env(),
            Cep18InitArgs {
                symbol: "PLS".to_string(),
                name: "Plascoin".to_string(),
                decimals: 100,
                initial_supply: 1_000_000_000.into(),
                minter_list: vec![],
                admin_list: vec![],
                modality: Some(1)
            }
        );
        let another_token_address = *another_token.address();

        // account to account
        test_approve_for(&mut cep18_token, owner, owner, alice);

        // todo: test approves for real contracts
        // account to contract
        test_approve_for(&mut cep18_token, owner, owner, another_token_address);

        // contract to contract
        test_approve_for(
            &mut cep18_token,
            token_address,
            token_address,
            another_token_address
        );
    }

    #[test]
    fn should_not_transfer_from_without_enough_allowance() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);

        // when the owner approves the spender to spend tokens on their behalf
        cep18_token.approve(&alice, &ALLOWANCE_AMOUNT_1.into());

        // then the allowance is set
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            ALLOWANCE_AMOUNT_1.into()
        );

        // and transferring more is not possible
        cep18_token.env().set_caller(alice);
        let result =
            cep18_token.try_transfer_from(&owner, &alice, &U256::from(ALLOWANCE_AMOUNT_1 + 1));
        assert_eq!(result.err().unwrap(), InsufficientAllowance.into());

        // but transferring less is possible
        cep18_token.transfer_from(&owner, &alice, &U256::from(ALLOWANCE_AMOUNT_1));
    }

    #[test]
    fn test_increase_allowance() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);

        // when the owner approves the spender to spend tokens on their behalf
        cep18_token.approve(&alice, &ALLOWANCE_AMOUNT_1.into());

        // then the allowance is set
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            ALLOWANCE_AMOUNT_1.into()
        );

        // when the owner decreases the allowance
        cep18_token.decrease_allowance(&alice, &ALLOWANCE_AMOUNT_2.into());

        // then the allowance is decreased
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            (ALLOWANCE_AMOUNT_1 - ALLOWANCE_AMOUNT_2).into()
        );

        // when the allowance is increased
        cep18_token.increase_allowance(&alice, &ALLOWANCE_AMOUNT_1.into());

        // then the allowance is increased
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            ((ALLOWANCE_AMOUNT_1 * 2) - ALLOWANCE_AMOUNT_2).into()
        );
    }

    #[test]
    fn should_transfer_full_owned_amount() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner transfers the full amount to alice
        cep18_token.transfer(&alice, &amount);

        // then the owner has no balance
        assert_eq!(cep18_token.balance_of(&owner), 0.into());

        // and alice has the full amount
        assert_eq!(cep18_token.balance_of(&alice), amount);
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_not_transfer_more_than_owned_balance() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner tries to transfer more than they have
        let result = cep18_token.try_transfer(&alice, &U256::from(amount + 1));

        // then the transfer fails
        assert_eq!(result.err().unwrap(), InsufficientBalance.into());

        // and the balances remain unchanged
        assert_eq!(cep18_token.balance_of(&owner), amount);
        assert_eq!(cep18_token.balance_of(&alice), 0.into());
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_transfer_from_account_to_account() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let transfer_amount = TRANSFER_AMOUNT_1.into();
        let allowance_amount = ALLOWANCE_AMOUNT_1.into();

        // when the owner approves the spender to spend tokens on their behalf
        cep18_token.approve(&alice, &allowance_amount);

        // then the allowance is set
        assert_eq!(cep18_token.allowance(&owner, &alice), allowance_amount);

        // when the spender spends the tokens
        cep18_token.env().set_caller(alice);
        cep18_token.transfer_from(&owner, &alice, &transfer_amount);

        // then the owner has less tokens
        assert_eq!(
            cep18_token.balance_of(&owner),
            U256::from(TOKEN_TOTAL_SUPPLY) - transfer_amount
        );

        // and alice has more tokens
        assert_eq!(cep18_token.balance_of(&alice), transfer_amount);

        // and the allowance is lowered
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            allowance_amount - transfer_amount
        );
    }

    #[test]
    fn should_transfer_from_account_by_contract() {
        // todo: prepare a contract and test it
    }

    #[test]
    fn should_not_be_able_to_own_transfer() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner tries to transfer to themselves
        let result = cep18_token.try_transfer(&owner, &amount);

        // then the transfer fails
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());

        // and the balances remain unchanged
        assert_eq!(cep18_token.balance_of(&owner), amount);
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_not_be_able_to_own_transfer_from() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner tries to approbve themselves
        let result = cep18_token.try_approve(&owner, &amount);

        // it fails
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());

        // when the owner tries to transfer from themselves
        let result = cep18_token.try_transfer_from(&owner, &owner, &amount);

        // then the transfer fails
        assert_eq!(result.err().unwrap(), CannotTargetSelfUser.into());

        // and the balances remain unchanged
        assert_eq!(cep18_token.balance_of(&owner), amount);
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_verify_zero_amount_transfer_is_noop() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner transfers zero tokens
        cep18_token.transfer(&alice, &U256::zero());

        // then the balances remain unchanged
        assert_eq!(cep18_token.balance_of(&owner), amount);
        assert_eq!(cep18_token.balance_of(&alice), 0.into());
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_verify_zero_amount_transfer_from_is_noop() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let amount = TOKEN_TOTAL_SUPPLY.into();

        // when the owner approves the spender to spend tokens on their behalf
        cep18_token.approve(&alice, &ALLOWANCE_AMOUNT_1.into());

        // when the owner transfers zero tokens from alice
        cep18_token.transfer_from(&owner, &alice, &U256::zero());

        // then the balances remain unchanged
        assert_eq!(cep18_token.balance_of(&owner), amount);
        assert_eq!(cep18_token.balance_of(&alice), 0.into());
        assert_eq!(cep18_token.total_supply(), amount);
    }

    #[test]
    fn should_transfer() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let another_token = setup_with_args(
            cep18_token.env(),
            Cep18InitArgs {
                symbol: "PLS".to_string(),
                name: "Plascoin".to_string(),
                decimals: 100,
                initial_supply: 1_000_000_000.into(),
                minter_list: vec![],
                admin_list: vec![],
                modality: Some(1)
            }
        );

        // when the owner transfers tokens to another token
        cep18_token.transfer(another_token.address(), &TRANSFER_AMOUNT_1.into());

        // then the balances are updated
        assert_eq!(
            cep18_token.balance_of(&owner),
            (TOKEN_TOTAL_SUPPLY - TRANSFER_AMOUNT_1).into()
        );
        assert_eq!(
            cep18_token.balance_of(another_token.address()),
            TRANSFER_AMOUNT_1.into()
        );

        // when the token transfers tokens to yet another token
        cep18_token.env().set_caller(*another_token.address());
        cep18_token.transfer(&cep18_token.address().clone(), &TRANSFER_AMOUNT_1.into());

        // then the balances are updated
        assert_eq!(cep18_token.balance_of(another_token.address()), 0.into());
        assert_eq!(
            cep18_token.balance_of(cep18_token.address()),
            TRANSFER_AMOUNT_1.into()
        );

        // when the token are transferred back to the original owner
        cep18_token.env().set_caller(cep18_token.address().clone());
        cep18_token.transfer(&owner, &TRANSFER_AMOUNT_1.into());

        // then the balances are updated
        assert_eq!(cep18_token.balance_of(&owner), TOKEN_TOTAL_SUPPLY.into());
        assert_eq!(cep18_token.balance_of(cep18_token.address()), 0.into());
        assert_eq!(cep18_token.balance_of(another_token.address()), 0.into());
    }
}
