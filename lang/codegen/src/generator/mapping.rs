use derive_more::From;
use odra_ir::MapExpr;
use proc_macro2::TokenStream;
use quote::quote;

use crate::GenerateCode;

#[derive(From)]
pub struct OdraMapping<'a> {
    item: &'a MapExpr
}

impl GenerateCode for OdraMapping<'_> {
    fn generate_code(&self) -> TokenStream {
        let root = &self.item.root_mapping;
        let init_root = quote!(let v = &#root;);
        let segments_count = self.item.segments.len();

        let value = self.item.segments.last().unwrap();

        if self.item.assign_token.is_some() && self.item.assigned_value.is_some() {
            let assigned_value = self.item.assigned_value.as_ref().unwrap();
            let dest_mapping_discovery = self
                .item
                .segments
                .iter()
                .take(segments_count - 1)
                .rev()
                .enumerate()
                .map(|(idx, e)| match idx {
                    0 => quote!(let mut v = v.get_instance(&#e);),
                    _ => quote!(let v = v.get_instance(&#e);)
                })
                .rev()
                .collect::<TokenStream>();
            let value_assign = quote!(v.set(&#value, #assigned_value));

            quote! {
                #init_root
                #dest_mapping_discovery
                #value_assign;
            }
        } else {
            let value_discovery = self
                .item
                .segments
                .iter()
                .take(segments_count - 1)
                .map(|e| quote!(let v = v.get_instance(&#e);))
                .collect::<TokenStream>();
            let return_value = quote!(odra::UnwrapOrRevert::unwrap_or_revert(v.get(&#value)));
            quote! {
                {
                    #init_root
                    #value_discovery
                    #return_value
                }
            }
        }
    }
}
