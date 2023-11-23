use odra::prelude::*;
use odra::{runtime_args, FromBytes, RuntimeArgs};
use odra::{CallDef, ContractEnv, HostEnv, Mapping, Variable};

pub struct Counter {
    env: Rc<ContractEnv>,
    count0: Variable<u32>,
    count1: Variable<u32>,
    count2: Variable<u32>,
    count3: Variable<u32>,
    count4: Variable<u32>,
    count5: Variable<u32>,
    count6: Variable<u32>,
    count7: Variable<u32>,
    count8: Variable<u32>,
    count9: Variable<u32>,
    counts: Mapping<u8, u32>
}

impl Counter {
    pub fn get_count(&self, index: u8) -> u32 {
        match index {
            0 => self.count0.get_or_default(),
            1 => self.count1.get_or_default(),
            2 => self.count2.get_or_default(),
            3 => self.count3.get_or_default(),
            4 => self.count4.get_or_default(),
            5 => self.count5.get_or_default(),
            6 => self.count6.get_or_default(),
            7 => self.count7.get_or_default(),
            8 => self.count8.get_or_default(),
            9 => self.count9.get_or_default(),
            _ => unreachable!()
        }
    }

    pub fn increment(&mut self, index: u8) {
        match index {
            0 => increment(&mut self.count0),
            1 => increment(&mut self.count1),
            2 => increment(&mut self.count2),
            3 => increment(&mut self.count3),
            4 => increment(&mut self.count4),
            5 => increment(&mut self.count5),
            6 => increment(&mut self.count6),
            7 => increment(&mut self.count7),
            8 => increment(&mut self.count8),
            9 => increment(&mut self.count9),
            _ => unreachable!()
        };
    }
}

fn increment(count: &mut Variable<u32>) {
    let a = count.get_or_default();
    count.set(a + 1);
}

mod __counter_pack_module {
    use super::*;
    impl odra::module::Module for Counter {
        fn new(env: Rc<ContractEnv>) -> Self {
            let count0 = Variable::new(Rc::clone(&env), 0);
            let count1 = Variable::new(Rc::clone(&env), 1);
            let count2 = Variable::new(Rc::clone(&env), 2);
            let count3 = Variable::new(Rc::clone(&env), 3);
            let count4 = Variable::new(Rc::clone(&env), 4);
            let count5 = Variable::new(Rc::clone(&env), 5);
            let count6 = Variable::new(Rc::clone(&env), 6);
            let count7 = Variable::new(Rc::clone(&env), 7);
            let count8 = Variable::new(Rc::clone(&env), 8);
            let count9 = Variable::new(Rc::clone(&env), 9);
            let counts = Mapping::new(Rc::clone(&env), 10);
            Self {
                env,
                count0,
                count1,
                count2,
                count3,
                count4,
                count5,
                count6,
                count7,
                count8,
                count9,
                counts
            }
        }

        fn env(&self) -> Rc<ContractEnv> {
            self.env.clone()
        }
    }
}
