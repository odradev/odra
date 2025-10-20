#![allow(missing_docs)]

use odra::{
    host::{Deployer, HostRefLoader, NoArgs},
    prelude::*
};
use odra_examples::factory::counter::{Counter, CounterFactory};

fn main() {
    let env = odra_casper_livenet_env::env();

    env.set_gas(450_000_000_000u64);
    let mut factory_ref = CounterFactory::deploy(&env, NoArgs);
    env.set_gas(290_000_000_000u64);
    let address = factory_ref.factory(String::from("FirstCounterFromFactory"), 99);
    env.set_gas(2_500_000_000u64);
    let mut counter = Counter::load(&env, address);
    println!("Counter incremented.");
    counter.increment();
    println!("Counter value: {}", counter.value());
}
