#![feature(box_patterns)]

use ir::{host_ref_item::HostRefItem, ref_item::RefItem, ModuleIR};
use proc_macro::TokenStream;
use proc_macro_error::{abort_if_dirty, proc_macro_error};

mod ir;
mod syn_utils;
#[cfg(test)]
mod test_utils;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match ModuleIR::try_from(&item.into()) {
        Ok(module) => {
            let code = module.self_code();
            let ref_item = RefItem::new(&module);
            let host_ref_item = HostRefItem::new(&module);

            let result = quote::quote! {
                #code
                #ref_item
            };
            abort_if_dirty();
            result
        }
        Err(e) => e.to_compile_error()
    }
    .into()
}
