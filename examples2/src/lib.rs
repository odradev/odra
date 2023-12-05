#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

pub mod counter;
pub mod counter_pack;
pub mod erc20;
pub mod reentrancy_guard;
