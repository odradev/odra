use crate::erc1155::errors::Error;
use crate::erc1155::events::{ApprovalForAll, TransferBatch, TransferSingle};
use crate::erc1155::Erc1155;
use crate::erc1155_receiver::Erc1155ReceiverRef;
use odra::contract_env::{caller, revert};
use odra::types::address::OdraAddress;
use odra::types::event::OdraEvent;
use odra::types::{Address, Bytes, U256};
use odra::Mapping;

/// The ERC1155 base implementation.
#[odra::module(events = [ApprovalForAll, TransferBatch, TransferSingle])]
pub struct Erc1155Base {
    pub balances: Mapping<(Address, U256), U256>,
    pub approvals: Mapping<(Address, Address), bool>
}

impl Erc1155 for Erc1155Base {
    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.balances.get_or_default(&(owner, id))
    }

    fn balance_of_batch(&self, owners: Vec<Address>, ids: Vec<U256>) -> Vec<U256> {
        if owners.len() != ids.len() {
            revert(Error::AccountsAndIdsLengthMismatch);
        }

        // get balances for each owner and id
        let mut balances = Vec::new();
        for i in 0..owners.len() {
            let balance = self.balance_of(owners[i], ids[i]);
            balances.push(balance);
        }

        balances
    }

    fn set_approval_for_all(&mut self, operator: Address, approved: bool) {
        let owner = caller();
        if owner == operator {
            revert(Error::ApprovalForSelf);
        }

        self.approvals.set(&(owner, operator), approved);

        ApprovalForAll {
            owner,
            operator,
            approved
        }
        .emit();
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.approvals.get_or_default(&(owner, operator))
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
        data: Option<Bytes>
    ) {
        let caller = caller();
        self.assert_approved_or_owner(caller, from);

        let from_balance = self.balance_of(from, id);
        if from_balance < amount {
            revert(Error::InsufficientBalance);
        }

        self.balances.set(&(from, id), from_balance - amount);
        self.balances
            .set(&(to, id), self.balance_of(to, id) + amount);

        TransferSingle {
            operator: Some(caller),
            from: Some(from),
            to: Some(to),
            id,
            value: amount
        }
        .emit();

        self.safe_transfer_acceptance_check(caller, from, to, id, amount, data);
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Option<Bytes>
    ) {
        let caller = caller();
        self.assert_approved_or_owner(caller, from);

        if ids.len() != amounts.len() {
            revert(Error::IdsAndAmountsLengthMismatch);
        }

        // batch transfer
        for i in 0..ids.len() {
            let id = ids[i];
            let amount = amounts[i];

            let from_balance = self.balance_of(from, id);
            if from_balance < amount {
                revert(Error::InsufficientBalance);
            }

            self.balances.set(&(from, id), from_balance - amount);
            self.balances
                .set(&(to, id), self.balance_of(to, id) + amount);
        }

        TransferBatch {
            operator: Some(caller),
            from: Some(from),
            to: Some(to),
            ids: ids.clone(),
            values: amounts.clone()
        }
        .emit();

        self.safe_batch_transfer_acceptance_check(caller, from, to, ids, amounts, data);
    }
}

impl Erc1155Base {
    fn is_approved_or_owner(&self, spender: Address, owner: Address) -> bool {
        let spender_is_owner = spender == owner;
        let spender_is_approved = self.is_approved_for_all(owner, spender);

        spender_is_owner || spender_is_approved
    }

    fn assert_approved_or_owner(&self, spender: Address, owner: Address) {
        if !self.is_approved_or_owner(spender, owner) {
            revert(Error::NotAnOwnerOrApproved);
        }
    }

    pub fn safe_transfer_acceptance_check(
        &self,
        operator: Address,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
        data: Option<Bytes>
    ) {
        if to.is_contract() {
            let response =
                Erc1155ReceiverRef::at(to).on_erc1155_received(operator, from, id, amount, data);
            if !response {
                revert(Error::TransferRejected);
            }
        }
    }

    pub fn safe_batch_transfer_acceptance_check(
        &self,
        operator: Address,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Option<Bytes>
    ) {
        if to.is_contract() {
            let response = Erc1155ReceiverRef::at(to)
                .on_erc1155_batch_received(operator, from, ids, amounts, data);
            if !response {
                revert(Error::TransferRejected);
            }
        }
    }
}
