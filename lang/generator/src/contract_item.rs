use odra_ir::contract_item::{
    contract_impl::ContractImpl, contract_struct::ContractStruct, ContractItem,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

pub fn generate_code(item: ContractItem) -> TokenStream {
    if let Some(item_struct) = item.contract_struct() {
        return generate_struct_code(item_struct);
    }

    let contract_impl = item.contract_impl().unwrap();
    generate_impl_code(contract_impl)
}

fn generate_struct_code(item_struct: &ContractStruct) -> TokenStream {
    item_struct.to_token_stream()
}

fn generate_impl_code(contract_impl: &ContractImpl) -> TokenStream {
    let original_impl = contract_impl.original_impl();
    let contract_def = generate_contract_def(contract_impl);
    let deploy = generate_deploy(contract_impl);
    let contract_ref = generate_contract_ref(contract_impl);

    quote! {
        #original_impl

        #contract_def

        #deploy

        #contract_ref
    }
}

fn generate_contract_def(contract_impl: &ContractImpl) -> TokenStream {
    let struct_ident = contract_impl.ident();
    let struct_name = struct_ident.to_string();

    let entrypoints = contract_impl
        .entrypoints()
        .iter()
        .map(|entrypoint| {
            let name = &entrypoint.ident.to_string();
            let args = &entrypoint
                .args
                .iter()
                .map(|arg| {
                    let name = &*arg.pat;
                    let ty = &*arg.ty;
                    let ty = quote!(<#ty as odra::types::CLTyped>::cl_type());
                    quote! {
                        odra::contract_def::Argument {
                            ident: String::from(stringify!(#name)),
                            ty: #ty,
                        },
                    }
                })
                .flatten()
                .collect::<TokenStream>();
            let ret = match &entrypoint.ret {
                syn::ReturnType::Default => quote!(odra::types::CLType::Unit),
                syn::ReturnType::Type(_, ty) => quote!(<#ty as odra::types::CLTyped>::cl_type()),
            };
            quote! {
                odra::contract_def::Entrypoint {
                    ident: String::from(#name),
                    args: vec![#args],
                    ret: #ret
                },
            }
        })
        .flatten()
        .collect::<TokenStream>();

    quote! {
        impl odra::contract_def::HasContractDef for #struct_ident {
            fn contract_def() -> odra::contract_def::ContractDef {
                odra::contract_def::ContractDef {
                    ident: String::from(#struct_name),
                    entrypoints: vec![
                        #entrypoints
                    ],
                }
            }
        }
    }
}

fn generate_deploy(contract_impl: &ContractImpl) -> TokenStream {
    let struct_ident = contract_impl.ident();
    let struct_name = struct_ident.to_string();
    let struct_name_lowercased = struct_name.to_lowercase();
    let ref_ident = format_ident!("{}Ref", struct_ident);

    let register_entrypoints = contract_impl
        .entrypoints()
        .iter()
        .map(|entrypoint| {
            let ident = &entrypoint.ident;
            let name = quote!(stringify!(#ident).to_string());
            let return_value = match &entrypoint.ret {
                syn::ReturnType::Default => quote!(None),
                syn::ReturnType::Type(_, _) => quote! {
                    let bytes = odra::types::bytesrepr::ToBytes::to_bytes(&result).unwrap();
                    Some(odra::types::bytesrepr::Bytes::from(bytes))
                }
            };
            let args = &entrypoint.args.iter().map(|arg| {
                let pat = &*arg.pat;
                quote!(args.get(stringify!(#pat)).cloned().unwrap().into_t().unwrap(),)
            }).flatten().collect::<TokenStream>();

            quote! {
                container.add(#name, |name, args| {
                    let instance = <#struct_ident as odra::instance::Instance>::instance(name.as_str());
                    let result = instance.#ident(#args);
                    #return_value
                });
            }
        })
        .flatten()
        .collect::<TokenStream>();

    quote! {
        #[cfg(all(test, feature = "wasm-test"))]
        impl #struct_ident {
            fn deploy(name: &str) -> #ref_ident {
                let container = odra::ContractContainer {
                    name: #struct_name_lowercased.to_string(),
                    wasm_path: format!("{}.wasm", #struct_name_lowercased),
                };

                let address = odra::TestEnv::register_contract(&container);
                #ref_ident { address }
            }
        }

        #[cfg(all(test, feature = "mock-vm"))]
        impl #struct_ident {
            fn deploy(name: &str) -> #ref_ident {
                let mut container = odra::ContractContainer {
                    name: name.to_string(),
                    entrypoints: Default::default(),
                };

                #register_entrypoints

                let address = odra::TestEnv::register_contract(&container);
                #ref_ident { address }
            }
        }
    }
}

fn generate_contract_ref(contract_impl: &ContractImpl) -> TokenStream {
    let struct_ident = contract_impl.ident();
    let ref_ident = format_ident!("{}Ref", struct_ident);

    let ref_entrypoints = contract_impl
        .entrypoints()
        .iter()
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = &entrypoint.ident.to_string();

            let args = match &entrypoint.args.is_empty() {
                true => quote! { RuntimeArgs::new()},
                false => {
                    let mut args = quote!(let mut args = RuntimeArgs::new(););
                    args.append_all(entrypoint.args.clone().into_iter().map(|arg| {
                        let pat = &*arg.pat;
                        quote! { args.insert(stringify!(#pat), #pat).unwrap(); }
                    }));
                    args.extend(quote!(args));
                    args
                },
            };

            let args = quote! {
                use odra::types::RuntimeArgs;
                let args = {
                    #args
                };
            };
            let fn_body = match &entrypoint.ret {
                syn::ReturnType::Default => quote! {
                    #args
                    odra::call_contract::<()>(&self.address, #entrypoint_name, &args);
                },
                syn::ReturnType::Type(_, _) => quote! {
                    #args
                    use odra::types::CLTyped;
                    odra::call_contract(&self.address, #entrypoint_name, &args)
                }
            };

            quote! {
                #sig {
                    #fn_body
                }
            }
        })
        .flatten()
        .collect::<TokenStream>();

    quote! {
        #[cfg(test)]
        pub struct #ref_ident {
            pub address: odra::types::Address,
        }

        #[cfg(test)]
        impl #ref_ident {
            #ref_entrypoints
        }
    }
}
