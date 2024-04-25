#[cfg(test)]
mod mint_and_burn_tests {
    use alloc::string::ToString;
    use alloc::vec;

    use odra::casper_types::U256;
    use odra::host::HostRef;
    use odra::ExecutionError::AdditionOverflow;

    use crate::cep18::errors::Error::{InsufficientBalance, InsufficientRights, MintBurnDisabled};
    use crate::cep18::utils::Cep18Modality;
    use crate::cep18_token::tests::{
        setup, setup_with_args, TOKEN_DECIMALS, TOKEN_NAME, TOKEN_OWNER_AMOUNT_1,
        TOKEN_OWNER_AMOUNT_2, TOKEN_SYMBOL, TOKEN_TOTAL_SUPPLY, TRANSFER_AMOUNT_1
    };
    use crate::cep18_token::Cep18InitArgs;

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
        assert_eq!(result, Err(AdditionOverflow.into()));
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
            modality: Some(Cep18Modality::MintAndBurn)
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
            modality: Some(Cep18Modality::MintAndBurn)
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
}
