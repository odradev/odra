pub mod bdd_env;
mod odra_world;
pub mod steps;
pub mod types;
pub mod utils;
mod virtual_balances;

use cucumber::{codegen::WorldInventory, World};
use std::fmt::Debug;

pub fn run<W: World + Debug + WorldInventory>(path: &str) {
    futures::executor::block_on(W::run(path));
}
