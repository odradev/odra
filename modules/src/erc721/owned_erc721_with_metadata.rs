use odra::types::{Address, Bytes, U256};

pub trait OwnedErc721WithMetadata {
    // Metadata
    fn init(&mut self, name: String, symbol: String, base_uri: String);
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn base_uri(&self) -> String;

    // Base
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
    fn approve(&mut self, approved: Address, token_id: U256);
    fn set_approval_for_all(&mut self, operator: Address, approved: bool);
    fn get_approved(&self, token_id: U256) -> Option<Address>;
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;

    // Ownable
    fn renounce_ownership(&mut self);
    fn transfer_ownership(&mut self, new_owner: Option<Address>);
    fn owner(&self) -> Option<Address>;
}
