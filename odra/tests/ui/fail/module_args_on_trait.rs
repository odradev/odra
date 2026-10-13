#[odra::module(events = [u32])]
pub trait Interface {
    fn value(&self) -> u32;
}

fn main() {}
