use crate::erc721::errors::Error;
use crate::erc721::events::{Approval, ApprovalForAll, Transfer};
use crate::erc721::Erc721;
use odra::contract_env::{caller, revert};
use odra::types::address::OdraAddress;
use odra::types::event::OdraEvent;
use odra::types::{Address, Bytes, CallArgs, U256};
use odra::{call_contract, Mapping, UnwrapOrRevert};

#[odra::module]
pub struct Erc721Base {
    // Erc721 base fields.
    pub balances: Mapping<Address, U256>,
    pub owners: Mapping<U256, Option<Address>>,
    pub token_approvals: Mapping<U256, Option<Address>>,
    pub operator_approvals: Mapping<(Address, Address), bool>
}

impl Erc721 for Erc721Base {
    fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get_or_default(&owner)
    }

    fn owner_of(&self, token_id: U256) -> Address {
        self.owners
            .get(&token_id)
            .unwrap_or_revert_with(Error::InvalidTokenId)
            .unwrap_or_revert_with(Error::InvalidTokenId)
    }

    fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        if !self.is_approved_or_owner(caller(), token_id) {
            revert(Error::NotAnOwnerOrApproved);
        }
        self.safe_transfer(from, to, token_id, None);
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes
    ) {
        if !self.is_approved_or_owner(caller(), token_id) {
            revert(Error::NotAnOwnerOrApproved);
        }
        self.safe_transfer(from, to, token_id, Some(data));
    }

    fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) {
        if !self.is_approved_or_owner(caller(), token_id) {
            revert(Error::NotAnOwnerOrApproved);
        }
        self.transfer(from, to, token_id);
    }

    fn approve(&mut self, approved: Option<Address>, token_id: U256) {
        let owner = self.owner_of(token_id);
        let caller = caller();

        if Some(owner) == approved {
            revert(Error::ApprovalToCurrentOwner);
        }

        if caller != owner && !self.is_approved_for_all(owner, caller) {
            revert(Error::NotAnOwnerOrApproved);
        }

        self.token_approvals.set(&token_id, approved);

        Approval {
            owner,
            approved,
            token_id
        }
        .emit();
    }

    fn set_approval_for_all(&mut self, operator: Address, approved: bool) {
        let caller = caller();
        if caller == operator {
            revert(Error::ApproveToCaller)
        }

        self.operator_approvals.set(&(caller, operator), approved);
        ApprovalForAll {
            owner: caller,
            operator,
            approved
        }
        .emit();
    }

    fn get_approved(&self, token_id: U256) -> Option<Address> {
        self.assert_exists(&token_id);
        self.token_approvals.get(&token_id).unwrap_or_default()
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operator_approvals
            .get(&(owner, operator))
            .unwrap_or(false)
    }
}

impl Erc721Base {
    pub fn is_approved_or_owner(&self, spender: Address, token_id: U256) -> bool {
        let owner = self.owner_of(token_id);
        (spender == owner)
            || self.get_approved(token_id) == Some(spender)
            || self.is_approved_for_all(owner, spender)
    }

    fn safe_transfer(&mut self, from: Address, to: Address, token_id: U256, data: Option<Bytes>) {
        self.transfer(from, to, token_id);
        if to.is_contract() {
            let mut call_args = CallArgs::new();
            call_args.insert("operator", caller());
            call_args.insert("from", from);
            call_args.insert("token_id", token_id);
            call_args.insert("data", data);

            let response: bool = call_contract(to, "on_erc721_received", call_args, None);

            if !response {
                revert(Error::TransferFailed)
            }
        }
    }

    fn transfer(&mut self, from: Address, to: Address, token_id: U256) {
        self.clear_approval(token_id);
        self.balances.set(&from, self.balance_of(from) - 1);
        self.balances.set(&to, self.balance_of(to) + 1);
        self.owners.set(&token_id, Some(to));

        Transfer {
            from: Some(from),
            to: Some(to),
            token_id
        }
        .emit();
    }

    pub fn clear_approval(&mut self, token_id: U256) {
        if self.token_approvals.get_or_default(&token_id).is_some() {
            self.token_approvals.set(&token_id, None);
        }
    }

    pub fn exists(&self, token_id: &U256) -> bool {
        self.owners.get(token_id).is_some()
    }

    pub fn assert_exists(&self, token_id: &U256) {
        if !self.exists(token_id) {
            revert(Error::InvalidTokenId);
        }
    }
}
