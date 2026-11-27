//! An entry point may spell its receiver out: `self: &Self` and `self: &mut Self` are `&self` and
//! `&mut self`. A `mut` binding of the receiver (`mut self: &Self`) does not make it mutable: the
//! mutability is the reference's.
#![deny(warnings)]

use odra::host::{Deployer, HostEnv, NoArgs};
use odra::prelude::*;

#[odra::module]
pub struct Counter {
    value: Var<u32>
}

#[odra::module]
impl Counter {
    pub fn get(self: &Self) -> u32 {
        self.value.get_or_default()
    }

    pub fn set(self: &mut Self, value: u32) {
        self.value.set(value);
    }

    #[allow(unused_mut)]
    pub fn get_through_mut_binding(mut self: &Self) -> u32 {
        self.value.get_or_default()
    }

    #[allow(unused_mut)]
    pub fn set_through_mut_binding(mut self: &mut Self, value: u32) {
        self.value.set(value);
    }

    #[odra(offchain)]
    pub fn doubled(self: &Self) -> u32 {
        2 * self.value.get_or_default()
    }
}

/// The host ref takes `&mut self` exactly for the entry points that change the state.
#[allow(dead_code)]
fn host_side(env: &HostEnv) {
    let mut counter = Counter::deploy(env, NoArgs);
    counter.set(1);
    counter.set_through_mut_binding(2);
    let counter = &counter;
    let _ = counter.get() + counter.get_through_mut_binding() + counter.doubled();
}

fn main() {}
