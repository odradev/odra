#[cfg(test)]
mod transfer_tests {
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, NoArgs};

    use crate::cep18::cep18_client_contract::Cep18ClientContractHostRef;
    use crate::cep18::errors::Error::{CannotTargetSelfUser, InsufficientBalance};
    use crate::cep18_token::tests::{
        setup, ALLOWANCE_AMOUNT_1, TOKEN_TOTAL_SUPPLY, TRANSFER_AMOUNT_1
    };

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
        let mut cep18_token = setup(false);
        let client_contract = Cep18ClientContractHostRef::deploy(cep18_token.env(), NoArgs);
        let spender = cep18_token.env().get_account(1);
        let owner = cep18_token.env().get_account(0);

        cep18_token.approve(client_contract.address(), &ALLOWANCE_AMOUNT_1.into());

        let spender_allowance_before = cep18_token.allowance(&owner, client_contract.address());
        let owner_balance_before = cep18_token.balance_of(&owner);

        client_contract.transfer_from_as_stored_contract(
            *cep18_token.address(),
            owner,
            spender,
            ALLOWANCE_AMOUNT_1.into()
        );

        assert_eq!(
            spender_allowance_before - ALLOWANCE_AMOUNT_1,
            cep18_token.allowance(&owner, &spender)
        );
        assert_eq!(
            owner_balance_before - ALLOWANCE_AMOUNT_1,
            cep18_token.balance_of(&owner)
        );
        assert_eq!(
            U256::from(ALLOWANCE_AMOUNT_1),
            cep18_token.balance_of(&spender)
        );
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
        let client_contract = Cep18ClientContractHostRef::deploy(cep18_token.env(), NoArgs);

        // when the owner transfers tokens to another contract
        cep18_token.transfer(client_contract.address(), &TRANSFER_AMOUNT_1.into());

        // then the balances are updated
        assert_eq!(
            cep18_token.balance_of(&owner),
            (TOKEN_TOTAL_SUPPLY - TRANSFER_AMOUNT_1).into()
        );
        assert_eq!(
            cep18_token.balance_of(client_contract.address()),
            TRANSFER_AMOUNT_1.into()
        );

        // when the token transfers tokens to yet another contract
        client_contract.transfer_as_stored_contract(
            *cep18_token.address(),
            *cep18_token.address(),
            TRANSFER_AMOUNT_1.into()
        );

        // then the balances are updated
        assert_eq!(cep18_token.balance_of(client_contract.address()), 0.into());
        assert_eq!(
            cep18_token.balance_of(cep18_token.address()),
            TRANSFER_AMOUNT_1.into()
        );
    }
}
