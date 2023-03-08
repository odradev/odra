pub mod erc721_base;
pub mod extensions;
pub mod owned_erc721_with_metadata;

use odra::types::{Address, Bytes, U256};

pub trait Erc721 {
    fn balance_of(&self, owner: Address) -> U256;
    fn owner_of(&self, token_id: U256) -> Address;
    fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256);
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes
    );
    fn transfer_from(&mut self, from: Address, to: Address, token_id: U256);
    fn approve(&mut self, approved: Option<Address>, token_id: U256);
    fn set_approval_for_all(&mut self, operator: Address, approved: bool);
    fn get_approved(&self, token_id: U256) -> Option<Address>;
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;
}

pub mod events {
    use odra::types::{Address, U256};

    #[derive(odra::Event)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub token_id: U256
    }

    #[derive(odra::Event)]
    pub struct Approval {
        pub(crate) owner: Address,
        pub(crate) approved: Option<Address>,
        pub(crate) token_id: U256
    }

    #[derive(odra::Event)]
    pub struct ApprovalForAll {
        pub(crate) owner: Address,
        pub(crate) operator: Address,
        pub(crate) approved: bool
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            InvalidTokenId => 30_000,
            NotAnOwnerOrApproved => 30_001,
            ApprovalToCurrentOwner => 30_002,
            ApproveToCaller => 30_003,
            NoSuchMethod => 30_004,
            TransferFailed => 30_005,
            StorageError => 30_501,
        }
    }
}
