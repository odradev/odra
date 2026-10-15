#[odra::module(events = [u32])]
pub trait Interface {
    fn value(&self) -> u32;
}

#[odra::module(factory = on)]
pub trait Factory {
    fn value(&self) -> u32;
}

fn main() {}
