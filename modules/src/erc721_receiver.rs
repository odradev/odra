//! A pluggable Odra module implementing Erc721Receiver.
use crate::erc721::extensions::erc721_receiver::Erc721Receiver as Erc721ReceiverTrait;
use crate::erc721::owned_erc721_with_metadata::OwnedErc721WithMetadata;
use crate::erc721_receiver::events::Received;
use crate::erc721_token::Erc721TokenContractRef;
use odra::casper_types::{bytesrepr::Bytes, U256};
use odra::prelude::*;
use odra::ContractRef;

/// The ERC721 receiver implementation.
#[odra::module]
pub struct Erc721Receiver;

#[odra::module]
impl Erc721ReceiverTrait for Erc721Receiver {
    fn on_erc721_received(
        &mut self,
        operator: &Address,
        from: &Address,
        token_id: &U256,
        data: &Option<Bytes>
    ) -> bool {
        self.env().emit_event(Received {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            data: data.clone()
        });
        Erc721TokenContractRef::new(self.env(), self.env().caller()).owner_of(token_id)
            == self.env().self_address()
    }
}

/// Erc721Receiver-related events.
pub mod events {
    use odra::casper_event_standard;
    use odra::casper_types::{bytesrepr::Bytes, U256};
    use odra::prelude::*;

    /// Emitted when the transfer is accepted by the contract.
    #[odra::event]
    pub struct Received {
        /// The operator that called the function.
        pub operator: Option<Address>,
        /// The address of of the sender.
        pub from: Option<Address>,
        /// The token id.
        pub token_id: U256,
        /// The token data.
        pub data: Option<Bytes>
    }
}
