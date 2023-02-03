use std::str::FromStr;

use odra_types::contract_def::{Argument, Entrypoint, EntrypointType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse2;

pub(crate) struct ContractEntrypoints<'a>(pub &'a Vec<Entrypoint>);

impl ToTokens for ContractEntrypoints<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens
            .extend(quote!(let mut entry_points = odra::casper::casper_types::EntryPoints::new();));
        tokens.append_all(self.0.iter().map(ContractEntrypoints::build_entry_point));
    }
}

impl ContractEntrypoints<'_> {
    fn build_entry_point(entrypoint: &Entrypoint) -> TokenStream {
        let entrypoint_ident = format_ident!("{}", entrypoint.ident);
        let params = EntrypointParams(&entrypoint.args);
        let token = TokenStream::from_str(&entrypoint.ret).unwrap();
        let ret = parse2::<syn::Type>(token).unwrap();
        let access = match &entrypoint.ty {
            EntrypointType::Constructor => quote! {
                odra::casper::casper_types::EntryPointAccess::Groups(vec![odra::casper::casper_types::Group::new("constructor")])
            },
            EntrypointType::Public => {
                quote! { odra::casper::casper_types::EntryPointAccess::Public }
            }
            EntrypointType::PublicPayable => {
                quote! { odra::casper::casper_types::EntryPointAccess::Public }
            }
        };
        quote! {
            entry_points.add_entry_point(
                odra::casper::casper_types::EntryPoint::new(
                    stringify!(#entrypoint_ident),
                    #params,
                    <#ret as odra::casper::casper_types::CLTyped>::cl_type(),
                    #access,
                    odra::casper::casper_types::EntryPointType::Contract,
                )
            );
        }
    }
}

struct EntrypointParams<'a>(pub &'a Vec<Argument>);

impl ToTokens for EntrypointParams<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            tokens.extend(quote!(Vec::<odra::casper::casper_types::Parameter>::new()));
        } else {
            let params_content = self
                .0
                .iter()
                .flat_map(|arg| {
                    let arg_ident = format_ident!("{}", arg.ident);
                    let token = TokenStream::from_str(&arg.ty).unwrap();
                    let ty = parse2::<syn::Type>(token).unwrap();
                    quote!(
                        params.push(
                            odra::casper::casper_types::Parameter::new(
                                stringify!(#arg_ident),
                                <#ty as odra::casper::casper_types::CLTyped>::cl_type()
                            )
                        );
                    )
                })
                .collect::<TokenStream>();

            let params = quote! {
                {
                    let mut params: Vec<odra::casper::casper_types::Parameter> = Vec::new();
                    #params_content
                    params
                }
            };

            tokens.extend(params);
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_eq_tokens;

    #[test]
    fn parse_cl_type() {
        let a = vec![Entrypoint {
            ident: String::from("call_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: String::from("i32")
            }],
            ret: String::from("bool"),
            ty: EntrypointType::Public,
            is_mut: false
        }];
        let ep = ContractEntrypoints(&a);
        assert_eq_tokens(
            ep,
            quote! {
                let mut entry_points = odra::casper::casper_types::EntryPoints::new();
                entry_points.add_entry_point(
                    odra::casper::casper_types::EntryPoint::new(
                        stringify!(call_me),
                        {
                            let mut params: Vec<odra::casper::casper_types::Parameter> = Vec::new();
                            params.push(odra::casper::casper_types::Parameter::new(stringify!(value), odra::casper::casper_types::CLType::I32));
                            params
                        },
                        odra::casper::casper_types::CLType::Bool,
                        odra::casper::casper_types::EntryPointAccess::Public,
                        odra::casper::casper_types::EntryPointType::Contract,
                    )
                );
            }
        );
    }
}
