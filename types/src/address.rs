pub trait OdraAddress {
    /// Returns true if the address is a contract address.
    fn is_contract(&self) -> bool;
}
