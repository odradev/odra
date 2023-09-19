//! Odra module implementing Erc1155 core.
use crate::erc1155::errors::Error;
use crate::erc1155::events::{ApprovalForAll, TransferBatch, TransferSingle};
use crate::erc1155::Erc1155;
use crate::erc1155_receiver::Erc1155ReceiverRef;
use odra::types::casper_types::bytesrepr::Bytes;
use odra::types::casper_types::U256;
use odra::types::OdraAddress;
use odra::{
    contract_env::{caller, revert},
    prelude::vec::Vec,
    types::{event::OdraEvent, Address},
    Mapping
};

/// The ERC1155 base implementation.
#[odra::module(events = [ApprovalForAll, TransferBatch, TransferSingle])]
pub struct Erc1155Base {
    pub balances: Mapping<Address, Mapping<U256, U256>>,
    pub approvals: Mapping<Address, Mapping<Address, bool>>
}

impl Erc1155 for Erc1155Base {
    fn balance_of(&self, owner: &Address, id: &U256) -> U256 {
        self.balances.get_instance(owner).get_or_default(id)
    }

    fn balance_of_batch(&self, owners: &[Address], ids: &[U256]) -> Vec<U256> {
        if owners.len() != ids.len() {
            revert(Error::AccountsAndIdsLengthMismatch);
        }

        // get balances for each owner and id
        let mut balances = Vec::new();
        for i in 0..owners.len() {
            let balance = self.balance_of(&owners[i], &ids[i]);
            balances.push(balance);
        }

        balances
    }

    fn set_approval_for_all(&mut self, operator: &Address, approved: bool) {
        let owner = &caller();
        if owner == operator {
            revert(Error::ApprovalForSelf);
        }

        self.approvals.get_instance(owner).set(operator, approved);

        ApprovalForAll {
            owner: *owner,
            operator: *operator,
            approved
        }
        .emit();
    }

    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool {
        self.approvals.get_instance(owner).get_or_default(operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    ) {
        let caller = caller();
        self.assert_approved_or_owner(&caller, from);

        let from_balance = self.balance_of(from, id);
        if from_balance < *amount {
            revert(Error::InsufficientBalance);
        }

        self.balances
            .get_instance(from)
            .set(id, from_balance - *amount);
        self.balances
            .get_instance(to)
            .set(id, self.balance_of(to, id) + *amount);

        TransferSingle {
            operator: Some(caller),
            from: Some(*from),
            to: Some(*to),
            id: *id,
            value: *amount
        }
        .emit();

        // verify the recipient
        self.safe_transfer_acceptance_check(&caller, from, to, id, amount, data);
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        ids: &[U256],
        amounts: &[U256],
        data: &Option<Bytes>
    ) {
        let caller = caller();
        self.assert_approved_or_owner(&caller, from);

        if ids.len() != amounts.len() {
            revert(Error::IdsAndAmountsLengthMismatch);
        }

        // batch transfer
        for i in 0..ids.len() {
            let id = ids[i];
            let amount = amounts[i];

            // balance check - if a single transfer is incorrect the whole transaction reverts.
            let from_balance = self.balance_of(from, &id);
            if from_balance < amount {
                revert(Error::InsufficientBalance);
            }

            // update balance
            self.balances
                .get_instance(from)
                .set(&id, from_balance - amount);
            self.balances
                .get_instance(to)
                .set(&id, self.balance_of(to, &id) + amount);
        }

        TransferBatch {
            operator: Some(caller),
            from: Some(*from),
            to: Some(*to),
            ids: ids.to_vec(),
            values: amounts.to_vec()
        }
        .emit();

        // verify the recipient
        self.safe_batch_transfer_acceptance_check(&caller, from, to, ids, amounts, data);
    }
}

impl Erc1155Base {
    fn is_approved_or_owner(&self, spender: &Address, owner: &Address) -> bool {
        let spender_is_owner = spender == owner;
        let spender_is_approved = self.is_approved_for_all(owner, spender);

        spender_is_owner || spender_is_approved
    }

    fn assert_approved_or_owner(&self, spender: &Address, owner: &Address) {
        if !self.is_approved_or_owner(spender, owner) {
            revert(Error::NotAnOwnerOrApproved);
        }
    }

    /// If the recipient `to` is a contract, must be a Erc1155Receiver, otherwise the transaction
    /// reverts with (TransferRejected)[Error::TransferRejected] error.
    pub fn safe_transfer_acceptance_check(
        &self,
        operator: &Address,
        from: &Address,
        to: &Address,
        id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    ) {
        if to.is_contract() {
            let response =
                Erc1155ReceiverRef::at(to).on_erc1155_received(operator, from, id, amount, data);
            if !response {
                revert(Error::TransferRejected);
            }
        }
    }

    /// If the recipient `to` is a contract, must be a Erc1155Receiver, otherwise the transaction
    /// reverts with (TransferRejected)[Error::TransferRejected] error.
    pub fn safe_batch_transfer_acceptance_check(
        &self,
        operator: &Address,
        from: &Address,
        to: &Address,
        ids: &[U256],
        amounts: &[U256],
        data: &Option<Bytes>
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
