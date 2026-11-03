use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::{
    ir::{FnIR, FnType, ModuleImplIR},
    utils
};

/// Emits `impl odra::schema::SchemaEntrypoints for Xxx` outside wasm: one
/// `odra::schema::entry_point::<Ret>(name, description, is_mut, is_payable, args)` per entry
/// point, and `schema_offchain_entrypoints` when the module has offchain functions.
pub struct SchemaEntrypointsItem {
    module_ident: Ident,
    entry_points: Vec<SchemaEntryPoint>,
    offchain_entry_points: Vec<SchemaEntryPoint>
}

impl TryFrom<&ModuleImplIR> for SchemaEntrypointsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident()?,
            entry_points: module
                .onchain_functions()
                .into_iter()
                .map(SchemaEntryPoint::new)
                .collect(),
            offchain_entry_points: module
                .offchain_functions()
                .into_iter()
                .map(SchemaEntryPoint::new)
                .collect()
        })
    }
}

impl ToTokens for SchemaEntrypointsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let entry_points = &self.entry_points;
        let offchain_entry_points = &self.offchain_entry_points;
        let ty_entrypoint_vec =
            quote!(odra::prelude::vec::Vec<odra::schema::casper_contract_schema::Entrypoint>);

        // The trait has an empty default; only a module with offchain functions overrides it.
        let offchain_fn = (!offchain_entry_points.is_empty()).then(|| {
            quote! {
                fn schema_offchain_entrypoints() -> #ty_entrypoint_vec {
                    odra::prelude::vec![ #(#offchain_entry_points),* ]
                }
            }
        });

        tokens.extend(quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEntrypoints for #module_ident {
                fn schema_entrypoints() -> #ty_entrypoint_vec {
                    odra::prelude::vec![ #(#entry_points),* ]
                }
                #offchain_fn
            }
        });
    }
}

/// Emits the schema entry points of a factory: its constructor as a bare `init` (no arguments,
/// no description: a factory is installed with none) and its three factory functions.
pub struct FactorySchemaEntrypointsItem(SchemaEntrypointsItem);

impl TryFrom<&ModuleImplIR> for FactorySchemaEntrypointsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let bare_constructor = module
            .functions()
            .iter()
            .filter(|f| f.is_constructor())
            .map(|f| SchemaEntryPoint {
                name: f.name_str(),
                description: Description::Own(String::new()),
                return_type: return_type(f),
                is_mut: true,
                is_payable: false,
                args: vec![]
            })
            .collect::<Vec<_>>();
        let factory_fns = [
            module.factory_fn(),
            module.factory_upgrade_fn(),
            module.factory_batch_upgrade_fn()
        ];
        Ok(Self(SchemaEntrypointsItem {
            module_ident: module.module_ident()?,
            entry_points: bare_constructor
                .into_iter()
                .chain(factory_fns.iter().map(SchemaEntryPoint::new))
                .collect(),
            offchain_entry_points: vec![]
        }))
    }
}

impl ToTokens for FactorySchemaEntrypointsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

/// One entry point as the schema describes it.
struct SchemaEntryPoint {
    name: String,
    description: Description,
    return_type: syn::Type,
    is_mut: bool,
    is_payable: bool,
    /// Argument name and type, the `__cargo_purse` of a payable entry point included.
    args: Vec<(String, syn::Type)>
}

/// Where the description comes from.
enum Description {
    /// The function's own doc comment, joined into one line.
    Own(String),
    /// A `delegate!`-generated function with no docs of its own takes the description of the
    /// submodule's entry point at `target` (`self.a.b`); `fallback` is the generated
    /// "Delegated. See ..." text used when that one has none either.
    Delegated {
        target: syn::ExprField,
        fallback: String
    }
}

impl SchemaEntryPoint {
    fn new(f: &FnIR) -> Self {
        let docs = utils::syn::docs_description(&f.docs());
        let description = match f.inherits_description_from() {
            Some(target) => Description::Delegated {
                target: target.clone(),
                fallback: docs
            },
            None => Description::Own(docs)
        };
        let mut args = match f.fn_type() {
            // The batch upgrade argument is generic, so its schema type is given explicitly.
            FnType::FactoryBatchUpgrader => vec![(String::from("args"), utils::ty::bytes())],
            _ => f
                .raw_typed_args()
                .iter()
                .map(|syn::PatType { pat, ty, .. }| {
                    (
                        pat.to_token_stream().to_string(),
                        utils::ty::unreferenced_ty(ty)
                    )
                })
                .collect()
        };
        if f.is_payable() {
            args.push((
                String::from("__cargo_purse"),
                parse_quote!(odra::casper_types::URef)
            ));
        }
        Self {
            name: f.name_str(),
            description,
            return_type: return_type(f),
            is_mut: f.is_mut(),
            is_payable: f.is_payable(),
            args
        }
    }
}

impl ToTokens for SchemaEntryPoint {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            return_type,
            is_mut,
            is_payable,
            ..
        } = self;
        let description = match &self.description {
            Description::Own(text) => quote!(#text),
            Description::Delegated { target, fallback } => {
                let module = format_ident!("__module");
                let target = rebase_field(target, &module);
                quote! {
                    &odra::schema::delegated_description::<Self, _>(
                        |#module| &#target,
                        #name,
                        #fallback
                    )
                }
            }
        };
        let args = self
            .args
            .iter()
            .map(|(name, ty)| quote!(odra::schema::argument::<#ty>(#name)));

        tokens.extend(quote! {
            odra::schema::entry_point::<#return_type>(
                #name,
                #description,
                #is_mut,
                #is_payable,
                odra::prelude::vec![ #(#args),* ]
            )
        });
    }
}

/// The function's return type, `()` when it has none.
fn return_type(f: &FnIR) -> syn::Type {
    match f.return_type() {
        syn::ReturnType::Default => parse_quote!(()),
        syn::ReturnType::Type(_, ty) => *ty
    }
}

/// Rewrites a `delegate!` target such as `self.a.b` to `<base>.a.b`, so it can be used
/// inside a closure where `self` is not available.
fn rebase_field(expr: &syn::ExprField, base: &Ident) -> syn::Expr {
    let member = &expr.member;
    match &*expr.base {
        syn::Expr::Field(inner) => {
            let inner = rebase_field(inner, base);
            parse_quote!(#inner.#member)
        }
        _ => parse_quote!(#base.#member)
    }
}

#[cfg(test)]
mod tests;
