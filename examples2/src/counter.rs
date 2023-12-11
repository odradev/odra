use odra::prelude::*;
use odra::{
    runtime_args, Address, CallDef, ContractEnv, FromBytes, HostEnv, Mapping, Module, OdraType,
    RuntimeArgs, Variable
};

#[derive(OdraType)]
struct MyCounter {
    value: u32,
    creator: Address
}

#[derive(OdraType)]
enum MyEnum {
    VariantA,
    VariantB,
    Unknown
}

#[odra::module]
pub struct Counter {
    count0: Variable<MyCounter>,
    count1: Variable<MyCounter>,
    count2: Variable<MyCounter>,
    count3: Variable<MyCounter>,
    count4: Variable<MyCounter>,
    count5: Variable<MyCounter>,
    count6: Variable<MyCounter>,
    count7: Variable<MyCounter>,
    count8: Variable<MyCounter>,
    count9: Variable<MyCounter>,
    counts: Mapping<u8, MyCounter>
}

impl Counter {
    pub fn get_count(&self, index: u8) -> u32 {
        match index {
            0 => self.count0.get().map(|c| c.value).unwrap_or_default(),
            1 => self.count1.get().map(|c| c.value).unwrap_or_default(),
            2 => self.count2.get().map(|c| c.value).unwrap_or_default(),
            3 => self.count3.get().map(|c| c.value).unwrap_or_default(),
            4 => self.count4.get().map(|c| c.value).unwrap_or_default(),
            5 => self.count5.get().map(|c| c.value).unwrap_or_default(),
            6 => self.count6.get().map(|c| c.value).unwrap_or_default(),
            7 => self.count7.get().map(|c| c.value).unwrap_or_default(),
            8 => self.count8.get().map(|c| c.value).unwrap_or_default(),
            9 => self.count9.get().map(|c| c.value).unwrap_or_default(),
            _ => unreachable!()
        }
    }

    pub fn increment(&mut self, index: u8) {
        match index {
            0 => increment(&self.env(), &mut self.count0),
            1 => increment(&self.env(), &mut self.count1),
            2 => increment(&self.env(), &mut self.count2),
            3 => increment(&self.env(), &mut self.count3),
            4 => increment(&self.env(), &mut self.count4),
            5 => increment(&self.env(), &mut self.count5),
            6 => increment(&self.env(), &mut self.count6),
            7 => increment(&self.env(), &mut self.count7),
            8 => increment(&self.env(), &mut self.count8),
            9 => increment(&self.env(), &mut self.count9),
            _ => unreachable!()
        };
    }
}

fn increment(env: &ContractEnv, count: &mut Variable<MyCounter>) {
    if let Some(counter) = count.get() {
        count.set(MyCounter {
            value: counter.value + 1,
            creator: counter.creator
        });
    } else {
        count.set(MyCounter {
            value: 1,
            creator: env.caller()
        });
    }
}
