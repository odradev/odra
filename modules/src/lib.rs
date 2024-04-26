#![doc = "Odra's library of plug and play modules"]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

pub mod access;
pub mod cep18;
pub mod cep18_token;
pub mod erc1155;
pub mod erc1155_receiver;
pub mod erc1155_token;
pub mod erc20;
pub mod erc721;
pub mod erc721_receiver;
pub mod erc721_token;
pub mod security;
pub mod wrapped_native;
