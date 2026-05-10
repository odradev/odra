#![doc = "Odra's library of plug and play modules"]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

pub mod access;
pub mod cep18;
pub mod cep18_token;
pub mod cep95;
pub mod cep96;
#[cfg(feature = "eip712")]
mod eip712;
pub mod erc1155;
pub mod erc1155_receiver;
pub mod erc1155_token;
pub mod erc20;
#[cfg(feature = "eip712")]
pub mod erc2612;
#[cfg(feature = "eip712")]
pub mod erc3009;
pub mod erc721;
pub mod erc721_receiver;
pub mod erc721_token;
pub mod security;
pub mod wrapped_native;
