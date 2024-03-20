//! Odra module implementing Erc721 core.
use crate::erc721::errors::Error::{
    self, ApprovalToCurrentOwner, ApproveToCaller, InvalidTokenId, NotAnOwnerOrApproved,
    TransferFailed
};
use crate::erc721::events::{Approval, ApprovalForAll, Transfer};
use crate::erc721::extensions::erc721_receiver::Erc721Receiver;
use crate::erc721::Erc721;
use crate::erc721_receiver::Erc721ReceiverContractRef;
use odra::prelude::*;
use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    Address, ContractRef, Mapping, UnwrapOrRevert
};
/// The ERC721 base implementation.
#[odra::module(events = [Approval, ApprovalForAll, Transfer], errors = [Error])]
pub struct Erc721Base {
    /// The token balances.
    pub balances: Mapping<Address, U256>,
    /// The token owners.
    pub owners: Mapping<U256, Option<Address>>,
    /// The token approvals.
    pub token_approvals: Mapping<U256, Option<Address>>,
    /// The operator approvals.
    pub operator_approvals: Mapping<(Address, Address), bool>
}

impl Erc721 for Erc721Base {
    fn balance_of(&self, owner: &Address) -> U256 {
        self.balances.get_or_default(owner)
    }

    fn owner_of(&self, token_id: &U256) -> Address {
        self.owners
            .get(token_id)
            .unwrap_or_revert_with(&self.env(), InvalidTokenId)
            .unwrap_or_revert_with(&self.env(), InvalidTokenId)
    }

    fn safe_transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256) {
        if !self.is_approved_or_owner(&self.env().caller(), token_id) {
            self.env().revert(NotAnOwnerOrApproved);
        }
        self.safe_transfer(from, to, token_id, &None);
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &U256,
        data: &Bytes
    ) {
        if !self.is_approved_or_owner(&self.env().caller(), token_id) {
            self.env().revert(NotAnOwnerOrApproved);
        }
        self.safe_transfer(from, to, token_id, &Some(data.clone()));
    }

    fn transfer_from(&mut self, from: &Address, to: &Address, token_id: &U256) {
        if !self.is_approved_or_owner(&self.env().caller(), token_id) {
            self.env().revert(NotAnOwnerOrApproved);
        }
        self.transfer(from, to, token_id);
    }

    fn approve(&mut self, approved: &Option<Address>, token_id: &U256) {
        let owner = self.owner_of(token_id);
        let caller = self.env().caller();

        if &Some(owner) == approved {
            self.env().revert(ApprovalToCurrentOwner);
        }

        if caller != owner && !self.is_approved_for_all(&owner, &caller) {
            self.env().revert(NotAnOwnerOrApproved);
        }

        self.token_approvals.set(token_id, *approved);

        self.env().emit_event(Approval {
            owner,
            approved: *approved,
            token_id: *token_id
        });
    }

    fn set_approval_for_all(&mut self, operator: &Address, approved: bool) {
        let caller = self.env().caller();
        if &caller == operator {
            self.env().revert(ApproveToCaller)
        }

        self.operator_approvals.set(&(caller, *operator), approved);
        self.env().emit_event(ApprovalForAll {
            owner: caller,
            operator: *operator,
            approved
        });
    }

    fn get_approved(&self, token_id: &U256) -> Option<Address> {
        self.assert_exists(token_id);
        self.token_approvals.get(token_id).unwrap_or_default()
    }

    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool {
        self.operator_approvals
            .get(&(*owner, *operator))
            .unwrap_or_default()
    }
}

impl Erc721Base {
    /// Returns true if the `spender` is the owner or an operator of the `token_id` token.
    pub fn is_approved_or_owner(&self, spender: &Address, token_id: &U256) -> bool {
        let owner = &self.owner_of(token_id);
        (spender == owner)
            || self.get_approved(token_id) == Some(*spender)
            || self.is_approved_for_all(owner, spender)
    }

    fn safe_transfer(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &U256,
        data: &Option<Bytes>
    ) {
        self.transfer(from, to, token_id);
        if to.is_contract() {
            let response = Erc721ReceiverContractRef::new(self.env(), *to).on_erc721_received(
                &self.env().caller(),
                from,
                token_id,
                data
            );

            if !response {
                self.env().revert(TransferFailed)
            }
        }
    }

    fn transfer(&mut self, from: &Address, to: &Address, token_id: &U256) {
        self.clear_approval(token_id);
        self.balances.set(from, self.balance_of(from) - 1);
        self.balances.set(to, self.balance_of(to) + 1);
        self.owners.set(token_id, Some(*to));

        self.env().emit_event(Transfer {
            from: Some(*from),
            to: Some(*to),
            token_id: *token_id
        });
    }

    /// Revokes permission to transfer the `token_id` token.
    pub fn clear_approval(&mut self, token_id: &U256) {
        if self.token_approvals.get_or_default(token_id).is_some() {
            self.token_approvals.set(token_id, None);
        }
    }

    /// Returns true if the `token_id` token exists.
    pub fn exists(&self, token_id: &U256) -> bool {
        self.owners.get(token_id).is_some()
    }

    /// Reverts with [Error::InvalidTokenId] if the `token_id` token does not exist.
    pub fn assert_exists(&self, token_id: &U256) {
        if !self.exists(token_id) {
            self.env().revert(InvalidTokenId);
        }
    }
}
