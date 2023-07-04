use std::{ops::{Deref, DerefMut}};

use once_cell::sync::Lazy;

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

pub trait Module {
    fn new(/* keys */) -> Self;
    
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
}

pub struct ModuleInstance<T: Module> {
    pub module: Lazy<T, fn() -> T>,
}

impl<T: Module> ModuleInstance<T> {
    pub fn new() -> Self {
        Self {
            module: Lazy::new(|| T::new()),
        }
    }
}

impl<T: Module> Deref for ModuleInstance<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &(*self.module)
    }
}

impl<T: Module> DerefMut for ModuleInstance<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (*self.module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref_module() {
        struct Adder {}

        impl Adder {
            fn add(&self, a: u32, b: u32) -> u32 {
                a + b
            }
        }

        impl Module for Adder {
            fn new() -> Self {
                Adder {}
            }

            fn id(&self) -> ModuleId {
                ModuleId("adder")
            }
        }

        struct Multiplier {
            adder: ModuleInstance<Adder>,
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
            fn new() -> Self {
                Multiplier {
                    adder: ModuleInstance::new(),
                }
            }

            fn id(&self) -> ModuleId {
                ModuleId("multiplier")
            }

            fn kids(&self) -> Vec<ModuleId> {
                vec![ModuleId("adder")]
            }
        }
        
        let module = ModuleInstance::<Multiplier>::new();
        // let keys = module.keys();
        // let module = ModuleInstance::<Multiplier>::new_with_key(keys);

        let result = module.multiply(2, 3);
        assert_eq!(result, 5);
    }
}