//! Erc721 receiver.
use odra::casper_types::{bytesrepr::Bytes, U256};
use odra::prelude::Address;

/// The ERC721 receiver interface.
pub trait Erc721Receiver {
    /// This function is called at the end of a [safe_transfer_from](crate::erc721::Erc721::safe_transfer_from) or
    /// [safe_transfer_from_with_data](crate::erc721::Erc721::safe_transfer_from_with_data), after the balance has been updated.
    ///
    /// To accept the transfer, this must return true.
    fn on_erc721_received(
        &mut self,
        operator: &Address,
        from: &Address,
        token_id: &U256,
        data: &Option<Bytes>
    ) -> bool;
}
