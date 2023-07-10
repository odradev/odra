use std::ops::{Deref, DerefMut};

use once_cell::sync::{Lazy, OnceCell};

pub struct ModuleId(pub &'static str);

// OwnedContract
// - Ownable
// - OwnedString
//      - Ownable (is_linked = true)
// - OwnedErc20
//      - Ownable (is_linked = true)

// Erc20
//  - name
//  - symbol

pub trait Module: Sized {
    fn new(keys: &'static [&'static str]) -> Self;

    fn id(&self) -> ModuleId;
    // fn parent(&self) -> ModuleId {
    //     ModuleId("parrent")
    // }
    fn kids(&self) -> Vec<ModuleId> {
        vec![]
    }
    fn path(&self) -> Vec<ModuleId> {
        vec![]
    }
    // fn address(&self) -> Vec<u8> {
    //     vec![]
    // }

    // fn types(&self) -> Vec<String>;
    // fn functions(&self) -> Vec<String>;
    // fn events(&self) -> Vec<String>;
    // fn emit_event(&self, event: &str, args: &[u8]);
    fn save_data(&self, _data: &[u8]) {}
    fn load_data(&self) -> Vec<u8> {
        vec![]
    }
    fn get_caller(&self) -> ModuleId {
        ModuleId("test")
    }
    fn call_module(&self, _module: ModuleId, _function: &str, _args: &[u8]) -> Vec<u8> {
        vec![]
    }
    // If true then calling_module is done via the host, so it's another contract.
    fn is_remote(&self) -> bool {
        false
    }

    // To help with preloading keys.
    fn is_mapping(&self) -> bool {
        false
    }

    fn is_linked(&self) -> bool {
        false
    }

    fn new_instance(keys: &'static[&'static str]) -> ModuleInstance<Self> {
        ModuleInstance::new(keys)
    }
}

pub struct ModuleInstance<T: Module> {
    pub keys: &'static [&'static str],
    pub module: OnceCell<T>
}

impl<T: Module> ModuleInstance<T> {
    pub fn new(keys: &'static [&'static str]) -> Self {
        Self {
            keys,
            module: OnceCell::new()
        }
    }

    pub fn get(&self) -> &T {
        self.init_if_none();
        self.module.get().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.init_if_none();
        self.module.get_mut().unwrap()
    }

    fn init_if_none(&self) {
        if self.module.get().is_none() {
            let _ = self.module.set(T::new(self.keys));
        }
    }
}

impl<T: Module> Deref for ModuleInstance<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: Module> DerefMut for ModuleInstance<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

pub struct Var<T> {
    key: &'static str,
    phantom: std::marker::PhantomData<T>
}

impl<T> Var<T> {
    pub fn set(&mut self, _value: T) {}
    pub fn get(&self) -> T {
        unimplemented!()
    }
}

impl<T> Module for Var<T> {
    fn new(keys: &'static [&'static str]) -> Self {
        Var::<T> {
            key: keys[0],
            phantom: std::marker::PhantomData
        }
    }

    fn id(&self) -> ModuleId {
        ModuleId("var")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref_module() {
        struct Adder {
            value: ModuleInstance<Var<u32>>
        }

        impl Adder {
            fn init(&mut self, value: u32) {
                self.value.set(value);
            }

            fn add(&self, a: u32, b: u32) -> u32 {
                a + b
            }
        }

        impl Module for Adder {
            fn new(keys: &'static [&'static str]) -> Self {
                Adder {
                    value: ModuleInstance::new(keys)
                }
            }

            fn id(&self) -> ModuleId {
                ModuleId("adder")
            }


        }

        struct Multiplier {
            adder: ModuleInstance<Adder>
        }

        impl Multiplier {
            fn multiply(&self, a: u32, b: u32) -> u32 {
                let mut result = 0;
                for _ in 0..b {
                    result = self.adder.add(result, a);
                }
                result
            }
        }

        impl Module for Multiplier {
            fn new(keys: &'static [&'static str]) -> Self {
                Self {
                    adder: ModuleInstance::new(keys)
                }
            }

            fn id(&self) -> ModuleId {
                ModuleId("multiplier")
            }
        }

        let keys = &["adder_value"];
        let module = ModuleInstance::<Multiplier>::new(keys);
        let result = module.multiply(2, 3);
        assert_eq!(result, 6);
    }
}
