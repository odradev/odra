use crate::types::{OdraType, WasmType};
use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{
    Argument, ContractSchema, Entrypoint, NamedCLType, Type
};
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, FnArg, PatIdent, PatType};

pub fn client(contract_schema: &ContractSchema) -> TokenStream {
    let client_name = format_ident!("{}WasmClient", contract_schema.contract_name);
    let struct_definition = client_struct_def(&client_name);
    let implementation = contract_schema
        .entry_points
        .iter()
        .map(entry_point_def)
        .collect::<Vec<_>>();

    quote::quote! {
        #struct_definition

        #[wasm_bindgen]
        impl #client_name {
            #[wasm_bindgen(constructor)]
            pub fn new(
                #[wasm_bindgen(js_name = "wasmClient")] wasm_client: &odra_wasm_client::OdraWasmClient,
                address: odra_wasm_client::types::Address
            ) -> Self {
                #client_name {
                    wasm_client: wasm_client.clone(),
                    address
                }
            }

            #(#implementation)*
        }
    }
}

fn client_struct_def<T: ToTokens>(client_name: &T) -> TokenStream {
    quote::quote! {
        #[wasm_bindgen]
        pub struct #client_name {
            wasm_client: odra_wasm_client::OdraWasmClient,
            address: odra_wasm_client::types::Address
        }
    }
}

fn entry_point_def(ep: &Entrypoint) -> syn::ImplItemFn {
    let returns_value = ep.return_ty.0 != NamedCLType::Unit;
    let is_mut = ep.is_mutable;
    let is_payable = ep.arguments.iter().any(|arg| arg.name == "__cargo_purse");

    if is_mut && returns_value {
        panic!("Mutable entry points with return values are not supported");
    } else if returns_value {
        getter_impl(ep)
    } else if is_payable {
        payable_impl(ep)
    } else if is_mut {
        mutable_impl(ep)
    } else {
        getter_impl(ep)
    }
}

fn getter_impl(ep: &Entrypoint) -> syn::ImplItemFn {
    let entry_point_ident = format_ident!("{}", &ep.name);
    let entry_point_str = &ep.name;
    let js_name = ep.name.to_case(Case::Camel);
    let args = ep.arguments.iter().map(entry_point_arg).collect::<Vec<_>>();
    let rt_args = ep
        .arguments
        .iter()
        .map(runtime_arg)
        .collect::<Vec<TokenStream>>();
    let wasm_ty = WasmType::from(&ep.return_ty);
    let odra_ty = OdraType::from(&ep.return_ty);

    let desc = ep.description.as_deref().unwrap_or("");
    let docs = quote::quote!(#[doc = #desc]);

    parse_quote! {
        #docs
        #[wasm_bindgen(js_name = #js_name)]
        pub async fn #entry_point_ident(&self, #(#args),*) -> Result<#wasm_ty, JsError> {
            self
                .wasm_client
                .call_entry_point_with_proxy::<#wasm_ty, #odra_ty>(*self.address, #entry_point_str, odra_wasm_client::casper_types::runtime_args! {
                    #(#rt_args),*
                })
                .await
        }
    }
}

fn payable_impl(ep: &Entrypoint) -> syn::ImplItemFn {
    let mut arguments = ep
        .arguments
        .iter()
        .filter(|arg| arg.name != "__cargo_purse")
        .cloned()
        .collect::<Vec<_>>();
    arguments.push(Argument {
        name: String::from("attached_value"),
        description: Some(String::from("Amount of CSPR to attach to the call.")),
        ty: Type(NamedCLType::U512),
        optional: false
    });
    let entry_point_ident = format_ident!("{}", &ep.name);
    let entry_point_str = &ep.name;
    let js_name = ep.name.to_case(Case::Camel);
    let args = arguments.iter().map(entry_point_arg).collect::<Vec<_>>();
    let rt_args = arguments
        .iter()
        .map(runtime_arg)
        .collect::<Vec<TokenStream>>();

    let desc = ep.description.as_deref().unwrap_or("");
    let docs = quote::quote!(#[doc = #desc]);

    let args = args.into_iter().map(|arg| {
        if let FnArg::Typed(PatType { pat: box syn::Pat::Ident(PatIdent { ident, .. }), .. }) = &arg {
            if ident == "__cargo_purse" {
                parse_quote!(#[wasm_bindgen(js = "attachedValue")] attached_value: odra_wasm_client::types::U512)
            } else {
                arg
            }
        } else {
            arg
        }
    }).collect::<Vec<_>>();

    parse_quote! {
        #docs
        #[wasm_bindgen(js_name = #js_name)]
        pub async fn #entry_point_ident(&self, #(#args),*) -> Result<odra_wasm_client::cspr_click::TransactionResult, JsError> {
            self.wasm_client
                .call_payable_entry_point(
                    *self.address,
                    #entry_point_str,
                    odra_wasm_client::casper_types::runtime_args! { #(#rt_args),* },
                    *attached_value
                )
                .await
        }
    }
}

fn mutable_impl(ep: &Entrypoint) -> syn::ImplItemFn {
    let entry_point_ident = format_ident!("{}", &ep.name);
    let entry_point_str = &ep.name;
    let js_name = ep.name.to_case(Case::Camel);
    let args = ep.arguments.iter().map(entry_point_arg).collect::<Vec<_>>();
    let rt_args = ep
        .arguments
        .iter()
        .map(runtime_arg)
        .collect::<Vec<TokenStream>>();
    let desc = ep.description.as_deref().unwrap_or("");
    let docs = quote::quote!(#[doc = #desc]);

    parse_quote! {
        #docs
        #[wasm_bindgen(js_name = #js_name)]
        pub async fn #entry_point_ident(&self, #(#args),*) -> Result<odra_wasm_client::cspr_click::TransactionResult, JsError> {
            self.wasm_client
                .call_entry_point(
                    *self.address,
                    #entry_point_str,
                    odra_wasm_client::casper_types::runtime_args! {
                        #(#rt_args),*
                    }
                )
                .await
        }
    }
}

fn runtime_arg(arg: &Argument) -> TokenStream {
    let name = &arg.name;
    let ident = format_ident!("{}", name);
    let odra_ty = OdraType::from(&arg.ty);
    quote::quote! { #name => odra_wasm_client::types::IntoOdraValue::<#odra_ty>::into_odra_value(#ident)? }
}

fn entry_point_arg(fn_arg: &Argument) -> syn::FnArg {
    let arg_ident = format_ident!("{}", fn_arg.name);
    let js_name = fn_arg.name.to_case(Case::Camel);
    let ty = WasmType::from(&fn_arg.ty);
    parse_quote!(#[wasm_bindgen(js_name = #js_name)] #arg_ident: #ty)
}
