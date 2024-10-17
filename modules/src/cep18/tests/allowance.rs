#[cfg(test)]
mod allowance_tests {
    use core::ops::Add;

    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, NoArgs};
    use odra::prelude::*;

    use crate::cep18::cep18_client_contract::Cep18ClientContract;
    use crate::cep18::errors::Error::InsufficientAllowance;
    use crate::cep18_token::tests::{
        invert_address, setup, ALLOWANCE_AMOUNT_1, ALLOWANCE_AMOUNT_2, TRANSFER_AMOUNT_1
    };
    use crate::cep18_token::Cep18HostRef;

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
    fn should_approve_funds() {
        // given a token
        let mut cep18_token = setup(false);
        let owner = cep18_token.env().get_account(0);
        let alice = cep18_token.env().get_account(1);
        let token_address = *cep18_token.address();
        let client_contract = Cep18ClientContract::deploy(cep18_token.env(), NoArgs);
        let another_client_contract = Cep18ClientContract::deploy(cep18_token.env(), NoArgs);

        let client_contract_address = client_contract.address();
        let another_client_contract_address = another_client_contract.address();

        // account to account
        test_approve_for(&mut cep18_token, owner, owner, alice);

        // account to contract
        cep18_token.approve(client_contract_address, &ALLOWANCE_AMOUNT_1.into());
        assert_eq!(
            cep18_token.allowance(&owner, client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );

        client_contract.transfer_from_as_stored_contract(
            token_address,
            owner,
            *client_contract_address,
            ALLOWANCE_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.balance_of(client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );

        // contract to contract
        client_contract.approve_as_stored_contract(
            token_address,
            *another_client_contract_address,
            ALLOWANCE_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.allowance(client_contract_address, another_client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
        );

        another_client_contract.transfer_from_as_stored_contract(
            token_address,
            *client_contract_address,
            *another_client_contract_address,
            ALLOWANCE_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.balance_of(another_client_contract_address),
            ALLOWANCE_AMOUNT_1.into()
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
    fn test_decrease_allowance() {
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
}
