use odra2::prelude::*;
use odra2::types::{runtime_args, FromBytes, RuntimeArgs};
use odra2::{CallDef, ContractEnv, HostEnv, Mapping, Variable};

pub struct Counter {
    env: Rc<ContractEnv>,
    count0: Variable<0, u32>,
    count1: Variable<1, u32>,
    count2: Variable<2, u32>,
    count3: Variable<3, u32>,
    count4: Variable<4, u32>,
    count5: Variable<5, u32>,
    count6: Variable<6, u32>,
    count7: Variable<7, u32>,
    count8: Variable<8, u32>,
    count9: Variable<9, u32>,
    counts: Mapping<10, u8, u32>
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

fn increment<const N: u8>(count: &mut Variable<N, u32>) {
    let a = count.get_or_default();
    count.set(a + 1);
}

mod __counter_pack_module {
    use super::*;
    impl odra2::module::Module for Counter {
        fn new(env: Rc<ContractEnv>) -> Self {
            let count0 = Variable::new(Rc::clone(&env));
            let count1 = Variable::new(Rc::clone(&env));
            let count2 = Variable::new(Rc::clone(&env));
            let count3 = Variable::new(Rc::clone(&env));
            let count4 = Variable::new(Rc::clone(&env));
            let count5 = Variable::new(Rc::clone(&env));
            let count6 = Variable::new(Rc::clone(&env));
            let count7 = Variable::new(Rc::clone(&env));
            let count8 = Variable::new(Rc::clone(&env));
            let count9 = Variable::new(Rc::clone(&env));
            let counts = Mapping::new(Rc::clone(&env));
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
