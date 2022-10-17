use odra::contract_def::{Argument, Entrypoint, EntrypointType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

use super::ty::WrappedType;

pub(crate) struct ContractEntrypoints<'a>(pub &'a Vec<Entrypoint>);

impl ToTokens for ContractEntrypoints<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(let mut entry_points = casper_backend::backend::casper_types::EntryPoints::new();));
        tokens.append_all(self.0.iter().map(ContractEntrypoints::build_entry_point));
    }
}

impl ContractEntrypoints<'_> {
    fn build_entry_point(entrypoint: &Entrypoint) -> TokenStream {
        let entrypoint_ident = format_ident!("{}", entrypoint.ident);
        let params = EntrypointParams(&entrypoint.args);
        let ret = WrappedType(&entrypoint.ret);
        let access = match &entrypoint.ty {
            EntrypointType::Constructor => quote! {
                casper_backend::backend::casper_types::EntryPointAccess::Groups(vec![casper_backend::backend::casper_types::Group::new("constructor")])
            },
            EntrypointType::Public => {
                quote! { casper_backend::backend::casper_types::EntryPointAccess::Public }
            }
            EntrypointType::PublicPayable => {
                quote! { casper_backend::backend::casper_types::EntryPointAccess::Public }
            }
        };
        quote! {
            entry_points.add_entry_point(
                casper_backend::backend::casper_types::EntryPoint::new(
                    stringify!(#entrypoint_ident),
                    #params,
                    #ret,
                    #access,
                    casper_backend::backend::casper_types::EntryPointType::Contract,
                )
            );
        }
    }
}

struct EntrypointParams<'a>(pub &'a Vec<Argument>);

impl ToTokens for EntrypointParams<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            tokens.extend(quote!(Vec::<
                casper_backend::backend::casper_types::Parameter,
            >::new()));
        } else {
            let params_content = self
                .0
                .iter()
                .flat_map(|arg| {
                    let arg_ident = format_ident!("{}", arg.ident);
                    let ty = WrappedType(&arg.ty);
                    quote!(params.push(casper_backend::backend::casper_types::Parameter::new(stringify!(#arg_ident), #ty));)
                })
                .collect::<TokenStream>();

            let params = quote! {
                {
                    let mut params: Vec<casper_backend::backend::casper_types::Parameter> = Vec::new();
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
    use crate::codegen::assert_eq_tokens;
    use odra::types::CLType;

    #[test]
    fn parse_cl_type() {
        let a = vec![Entrypoint {
            ident: String::from("call_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: CLType::I32,
            }],
            ret: CLType::Bool,
            ty: EntrypointType::Public,
        }];
        let ep = ContractEntrypoints(&a);
        assert_eq_tokens(
            ep,
            quote! {
                let mut entry_points = casper_backend::backend::casper_types::EntryPoints::new();
                entry_points.add_entry_point(
                    casper_backend::backend::casper_types::EntryPoint::new(
                        stringify!(call_me),
                        {
                            let mut params: Vec<casper_backend::backend::casper_types::Parameter> = Vec::new();
                            params.push(casper_backend::backend::casper_types::Parameter::new(stringify!(value), casper_backend::backend::casper_types::CLType::I32));
                            params
                        },
                        casper_backend::backend::casper_types::CLType::Bool,
                        casper_backend::backend::casper_types::EntryPointAccess::Public,
                        casper_backend::backend::casper_types::EntryPointType::Contract,
                    )
                );
            },
        );
    }
}
