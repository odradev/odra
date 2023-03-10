use odra::types::{Address, Bytes, U256};

pub trait OwnedErc1155 {
    fn init(&mut self);

    // Erc1155 base
    fn balance_of(&self, owner: Address, id: U256) -> U256;
    fn balance_of_batch(&self, owners: Vec<Address>, ids: Vec<U256>) -> Vec<U256>;
    fn set_approval_for_all(&mut self, operator: Address, approved: bool);
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
        data: Bytes
    );
    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes
    );
    fn batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>
    );

    // Ownable
    fn renounce_ownership(&mut self);
    fn transfer_ownership(&mut self, new_owner: Option<Address>);
    fn owner(&self) -> Option<Address>;
}
