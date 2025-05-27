#[cfg(test)]
mod mint_and_burn_tests {
    use odra::prelude::*;

    use odra::casper_types::U256;
    use odra::host::HostRef;

    use crate::cep18::errors::Error::{InsufficientBalance, InsufficientRights};
    use crate::cep18_token::tests::{
        setup, TOKEN_OWNER_AMOUNT_1, TOKEN_OWNER_AMOUNT_2, TRANSFER_AMOUNT_1
    };

    #[test]
    fn test_mint_and_burn() {
        let mut cep18_token = setup();

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
        let mut cep18_token = setup();
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
        assert_eq!(result, Err(ExecutionError::AdditionOverflow.into()));
    }

    #[test]
    fn should_not_burn_above_balance() {
        let mut cep18_token = setup();
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
    fn test_security_no_rights() {
        // given a token with mint and burn enabled
        let mut cep18_token = setup();
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
}
