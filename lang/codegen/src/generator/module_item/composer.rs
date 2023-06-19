use derive_more::From;
use odra_ir::module::ModuleStruct;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma};

use crate::GenerateCode;

#[derive(From)]
pub struct ModuleComposer<'a> {
    module: &'a ModuleStruct
}

as_ref_for_contract_struct_generator!(ModuleComposer);

impl GenerateCode for ModuleComposer<'_> {
    fn generate_code(&self) -> TokenStream {
        let composer_ident = format_ident!("{}Composer", self.module.item.ident);
        let module_ident = &self.module.item.ident;

        let fields = match self.module.item.fields {
            syn::Fields::Named(ref fields) => fields,
            _ => panic!("ModuleComposer can only be generated for named fields")
        };

        let struct_fields = fields
            .named
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                quote! {
                    #field_ident: core::option::Option<#field_type>
                }
            })
            .collect::<Punctuated<TokenStream, Comma>>();

        let init_fields = fields.named.iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();

                quote! {
                    #field_ident: self.#field_ident.unwrap_or_else(|| odra::DynamicInstance::instance(&[&self.namespace, "#".as_bytes(), stringify!(#field_ident).as_bytes()].concat()))
                }
        }).collect::<Punctuated<TokenStream, Comma>>();

        let empty_fields = fields
            .named
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                quote! {
                    #field_ident: core::option::Option::None
                }
            })
            .collect::<Punctuated<TokenStream, Comma>>();

        let functions = fields
            .named
            .iter()
            .map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                let function_name = format_ident!("with_{}", field_ident);
                quote! {
                    pub fn #function_name(mut self, #field_ident: &#field_type) -> Self {
                        self.#field_ident = core::option::Option::Some(#field_ident.clone());
                        self
                    }
                }
            })
            .collect::<TokenStream>();

        quote! {
             pub struct #composer_ident {
                 namespace: Vec<u8>,
                 #struct_fields
             }

             impl #composer_ident {
                 pub fn new(namespace: &[u8], name: &str) -> Self {
                     Self {
                         namespace: [namespace, "#".as_bytes(), name.as_bytes()].concat(),
                         #empty_fields
                     }
                 }

                 #functions

                 pub fn compose(self) -> #module_ident {
                     #module_ident {
                         #init_fields
                     }
                 }
             }
        }
    }
}
