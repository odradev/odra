use odra::types::{Address, Bytes, U256};

pub mod erc1155_base;
pub mod extensions;
pub mod owned_erc1155;

/// The ERC-1155 interface as defined in the standard.
pub trait Erc1155 {
    fn balance_of(&self, owner: &Address, id: &U256) -> U256;
    fn balance_of_batch(&self, owners: &Vec<Address>, ids: &Vec<U256>) -> Vec<U256>;
    fn set_approval_for_all(&mut self, operator: &Address, approved: &bool);
    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool;
    fn safe_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    );
    fn safe_batch_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        ids: &Vec<U256>,
        amounts: &Vec<U256>,
        data: &Option<Bytes>
    );
}

pub mod events {
    use odra::types::{Address, U256};

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct TransferSingle {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub id: U256,
        pub value: U256
    }

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct TransferBatch {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub ids: Vec<U256>,
        pub values: Vec<U256>
    }

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct ApprovalForAll {
        pub owner: Address,
        pub operator: Address,
        pub approved: bool
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            AccountsAndIdsLengthMismatch => 30_000,
            ApprovalForSelf => 30_001,
            NotAnOwnerOrApproved => 30_002,
            InsufficientBalance => 30_003,
            TransferRejected => 30_004,
            IdsAndAmountsLengthMismatch => 30_005,
        }
    }
}
