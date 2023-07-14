use odra_casper_shared::consts;
use odra_types::contract_def::{Argument, Entrypoint, EntrypointType};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, Token};

use crate::ty::CasperType;

pub(crate) struct ContractEntrypoints<'a>(pub &'a [Entrypoint]);

impl ToTokens for ContractEntrypoints<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens
            .extend(quote!(let mut entry_points = odra::casper::casper_types::EntryPoints::new();));
        tokens.append_all(self.0.iter().map(ContractEntrypoints::build_entry_point));
    }
}

impl ContractEntrypoints<'_> {
    fn build_entry_point(entrypoint: &Entrypoint) -> TokenStream {
        let entrypoint_ident = &entrypoint.ident;
        let params = EntrypointParams(&entrypoint.args);
        let ret = CasperType(&entrypoint.ret);
        let constructor_group_name = consts::CONSTRUCTOR_GROUP_NAME;
        let access = match &entrypoint.ty {
            EntrypointType::Constructor { .. } => quote! {
                odra::casper::casper_types::EntryPointAccess::Groups(vec![odra::casper::casper_types::Group::new(#constructor_group_name)])
            },
            EntrypointType::Public { .. } => {
                quote! { odra::casper::casper_types::EntryPointAccess::Public }
            }
            EntrypointType::PublicPayable { .. } => {
                quote! { odra::casper::casper_types::EntryPointAccess::Public }
            }
        };
        quote! {
            entry_points.add_entry_point(
                odra::casper::casper_types::EntryPoint::new(
                    #entrypoint_ident,
                    #params,
                    #ret,
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
        let params_content = self
            .0
            .iter()
            .map(|arg| {
                let arg_ident = &arg.ident;
                let ty = CasperType(&arg.ty);
                quote!(odra::casper::casper_types::Parameter::new(#arg_ident, #ty))
            })
            .collect::<Punctuated<TokenStream, Token![,]>>();

        let params = quote!(vec![#params_content]);

        tokens.extend(params);
    }
}

#[cfg(test)]
mod test {
    use odra_types::Type;

    use super::*;
    use crate::assert_eq_tokens;

    #[test]
    fn parse_cl_type() {
        let a = vec![Entrypoint {
            ident: String::from("call_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: Type::I32,
                is_ref: false
            }],
            ret: Type::Bool,
            ty: EntrypointType::Public {
                non_reentrant: false
            },
            is_mut: false
        }];
        let ep = ContractEntrypoints(&a);
        assert_eq_tokens(
            ep,
            quote! {
                let mut entry_points = odra::casper::casper_types::EntryPoints::new();
                entry_points.add_entry_point(
                    odra::casper::casper_types::EntryPoint::new(
                        "call_me",
                        vec![odra::casper::casper_types::Parameter::new("value", odra::casper::casper_types::CLType::I32)],
                        odra::casper::casper_types::CLType::Bool,
                        odra::casper::casper_types::EntryPointAccess::Public,
                        odra::casper::casper_types::EntryPointType::Contract,
                    )
                );
            }
        );
    }
}
