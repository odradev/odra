cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        use std::collections::HashMap;
        use odra_types::{bytesrepr::Bytes, RuntimeArgs};

        type Fun = fn(String, RuntimeArgs) -> Option<Bytes>;

        #[derive(Default, Clone)]
        pub struct ContractContainer {
            pub name: String,
            pub wasm_path: String,
            pub entrypoints: HashMap<String, Fun>,
        }

        impl ContractContainer {
            pub fn add(&mut self, entrypoint: String, f: Fun) {
                self.entrypoints.insert(entrypoint, f);
            }

            pub fn call(&self, entrypoint: String, args: RuntimeArgs) -> Option<Bytes> {
                let f = self.entrypoints.get(&entrypoint).unwrap();
                f(self.name.clone(), args)
            }
        }
    } else if #[cfg(feature = "wasm-test")] {
        use odra_types::{bytesrepr::Bytes, RuntimeArgs};
        #[derive(Default, Clone)]
        pub struct ContractContainer {
            pub name: String,
            pub wasm_path: String,
        }
    } else {
        compile_error!("Unsupported feature");
    }
}
