//! Every error the macros report to the user, built in one place.
//!
//! A message names the offending token in backticks, says why it is rejected and, when there is
//! an obvious fix, says it after a semicolon. The span is the smallest token that is wrong: the
//! argument pattern, not the function; the attribute, not the item it sits on.
//!
//! Detection stays where the input is inspected; only the wording lives here, so the messages are
//! reviewed together and the module and factory paths report the same mistake the same way.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;

/// Collects every item, or every error that occurred, combined into one.
///
/// A `collect::<syn::Result<Vec<_>>>()` stops at the first error; this reports all of them in one
/// build, so the user fixes all the reserved names of an impl at once.
pub fn collect_all<T, I>(results: I) -> syn::Result<Vec<T>>
where
    I: IntoIterator<Item = syn::Result<T>>
{
    let mut items = Vec::new();
    let mut error: Option<syn::Error> = None;
    for result in results {
        match result {
            Ok(item) => items.push(item),
            Err(e) => match error.as_mut() {
                Some(error) => error.combine(e),
                None => error = Some(e)
            }
        }
    }
    match error {
        Some(error) => Err(error),
        None => Ok(items)
    }
}

/// Runs every check and reports all the errors as one.
pub fn check_all<I>(checks: I) -> syn::Result<()>
where
    I: IntoIterator<Item = syn::Result<()>>
{
    collect_all(checks).map(|_| ())
}

/// The value when it and the check both succeed; otherwise the errors of both, the value's first.
pub fn checked<T>(value: syn::Result<T>, check: syn::Result<()>) -> syn::Result<T> {
    match (value, check) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(mut error), Err(other)) => {
            error.combine(other);
            Err(error)
        }
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error)
    }
}

// --- What the attribute macro is applied to -------------------------------------------------

/// `#[odra::module]` on something that is neither a struct nor an impl or trait block.
pub fn module_on_other_item(item: &TokenStream) -> syn::Error {
    syn::Error::new(
        item_span(item),
        format!(
            "`#[odra::module]` goes on the module struct and on its impl or trait block; this is {}",
            tokens_kind(item)
        )
    )
}

/// `#[odra::factory]` on something that is neither a struct nor an impl or trait block.
pub fn factory_on_other_item(item: &TokenStream) -> syn::Error {
    syn::Error::new(
        item_span(item),
        format!(
            "`#[odra::factory]` goes on the factory struct and on its impl block; this is {}",
            tokens_kind(item)
        )
    )
}

/// `#[odra::odra_type]` or `#[odra::odra_error]` on something that is neither a struct nor an
/// enum.
pub fn type_on_other_item(item: &syn::Item) -> syn::Error {
    syn::Error::new(
        item_ident_span(item),
        format!(
            "`#[odra::odra_type]` and `#[odra::odra_error]` go on a struct or an enum; this is {}",
            item_kind(item)
        )
    )
}

/// `#[odra::odra_error]` on a struct.
pub fn error_on_struct(item: &syn::Item) -> syn::Error {
    syn::Error::new(
        item_ident_span(item),
        "`#[odra::odra_error]` goes on an enum, one variant per error; this is a struct"
    )
}

/// An error enum variant carrying data: `Overflow(u32)` or `Overflow { by: u32 }`.
pub fn error_variant_with_data(variant: &syn::Variant) -> syn::Error {
    syn::Error::new_spanned(
        variant,
        format!(
            "`{}` carries data; an error variant is a bare name, as an error is only its code",
            variant.ident
        )
    )
}

/// `#[odra::external_contract]` on an impl block.
pub fn external_contract_on_impl(item: &syn::ItemImpl) -> syn::Error {
    syn::Error::new_spanned(
        item.impl_token,
        "`#[odra::external_contract]` goes on a trait describing the contract's entry points, \
         not on an impl block"
    )
}

/// `#[odra::external_contract]` on something that is not a trait (the module macros dispatch
/// structs and impls before, so for them this is unreachable).
pub fn impl_or_trait_expected(item: &TokenStream) -> syn::Error {
    syn::Error::new(
        item_span(item),
        format!(
            "`#[odra::external_contract]` goes on a trait describing the contract's entry points; \
             this is {}",
            tokens_kind(item)
        )
    )
}

/// `#[derive(IntoRuntimeArgs)]` on an enum or a union.
pub fn runtime_args_on_non_struct(item: &syn::DeriveInput) -> syn::Error {
    syn::Error::new(
        item.ident.span(),
        "`#[derive(IntoRuntimeArgs)]` goes on a struct with named fields, one field per \
         argument; this is not a struct"
    )
}

/// `#[derive(IntoRuntimeArgs)]` on a tuple struct.
pub fn runtime_args_on_unnamed_fields(fields: &syn::Fields) -> syn::Error {
    syn::Error::new_spanned(
        fields,
        "`#[derive(IntoRuntimeArgs)]` needs named fields, as a field's name is the argument's \
         name; write `struct Args { name: Type }`"
    )
}

// --- #[odra::ref_helpers] ----------------------------------------------------------------------

pub fn ref_helpers_with_args(attr: &TokenStream) -> syn::Error {
    syn::Error::new_spanned(attr, "`#[odra::ref_helpers]` takes no arguments")
}

pub fn ref_helpers_on_non_impl(item: &TokenStream) -> syn::Error {
    syn::Error::new(
        item_span(item),
        "`#[odra::ref_helpers]` goes on an inherent impl block named after a module or an \
         external contract; this is not an impl block"
    )
}

pub fn ref_helpers_on_trait_impl(trait_path: &syn::Path) -> syn::Error {
    syn::Error::new_spanned(
        trait_path,
        "`#[odra::ref_helpers]` goes on an inherent impl block, not a trait impl; drop the trait"
    )
}

pub fn ref_helpers_with_generics(generics: &syn::Generics) -> syn::Error {
    syn::Error::new_spanned(
        generics,
        "`#[odra::ref_helpers]` does not support generics on the impl block"
    )
}

pub fn ref_helpers_on_non_path(self_ty: &syn::Type) -> syn::Error {
    syn::Error::new_spanned(
        self_ty,
        "`#[odra::ref_helpers]` expects the name of a module or of an external contract here"
    )
}

pub fn ref_helpers_with_generic_args(args: &syn::PathArguments) -> syn::Error {
    syn::Error::new_spanned(
        args,
        "`#[odra::ref_helpers]` does not support generic arguments on the module name"
    )
}

// --- #[odra::module(..)] arguments -------------------------------------------------------------

/// A `#[odra::module(..)]` argument that only the struct takes, given on the impl block.
pub fn module_arg_on_impl(name: &str, span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "`{name}` is not allowed on an impl block; put it on the module struct: \
             `#[odra::module({name} = ...)] pub struct ...`"
        )
    )
}

/// Any `#[odra::module(..)]` argument given on a trait.
pub fn module_arg_on_trait(name: &str, span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "`{name}` is not allowed on a trait; `#[odra::module]` on a trait takes no arguments"
        )
    )
}

/// A `#[odra::module(..)]` argument given twice.
pub fn module_arg_twice(name: &str, span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!("`{name}` is given twice; a `#[odra::module(..)]` argument is given at most once")
    )
}

/// Something in `#[odra::module(..)]` that is not one of its arguments.
pub fn unknown_module_arg(input: syn::parse::ParseStream, expected: &[&str]) -> syn::Error {
    let found = input
        .fork()
        .parse::<TokenStream>()
        .ok()
        .and_then(|rest| rest.into_iter().next().map(|t| t.to_string()));
    let found = found.unwrap_or_default();
    input.error(format!(
        "`{found}` is not a `#[odra::module(..)]` argument; expected one of: {}",
        expected.join(", ")
    ))
}

// --- #[odra(..)] on functions ------------------------------------------------------------------

pub fn odra_attr_malformed(attr: &syn::Attribute, expected: &str) -> syn::Error {
    syn::Error::new_spanned(
        attr,
        format!("malformed odra attribute, expected `#[odra(..)]` with one or more of: {expected}")
    )
}

pub fn odra_attr_empty(attr: &syn::Attribute, expected: &str) -> syn::Error {
    syn::Error::new_spanned(
        attr,
        format!("empty odra attribute, expected one or more of: {expected}")
    )
}

pub fn odra_attr_duplicate(meta: &syn::Meta, name: &str) -> syn::Error {
    syn::Error::new_spanned(meta, format!("duplicate odra attribute `{name}`"))
}

pub fn odra_attr_unknown(meta: &syn::Meta, name: &str, expected: &str) -> syn::Error {
    syn::Error::new_spanned(
        meta,
        format!("unknown odra attribute `{name}`, expected one of: {expected}")
    )
}

pub fn odra_attr_with_value(meta: &syn::Meta, name: &str) -> syn::Error {
    syn::Error::new_spanned(meta, format!("odra attribute `{name}` takes no value"))
}

/// An `#[odra(..)]` attribute on `init` or `upgrade`, whose entry points ignore it.
pub fn odra_attr_on_special_fn(attr: &syn::Attribute, arg: &str, fn_name: &str) -> syn::Error {
    let reason = match arg {
        "payable" => format!(
            "`{fn_name}` cannot receive attached tokens - there is no way to attach CSPR to it. \
             Use a separate payable entrypoint instead"
        ),
        _ => format!("it is not applied when generating the `{fn_name}` entrypoint")
    };
    syn::Error::new_spanned(
        attr,
        format!("`#[odra({arg})]` is not allowed on `{fn_name}`: {reason}")
    )
}

/// Why `#[odra(offchain)]` does not fit the function it is on.
pub enum OffchainMisuse {
    InTraitImpl,
    InFactory,
    MutReceiver,
    PayableOrNonReentrant,
    NoReceiver
}

/// `#[odra(offchain)]` on a function it does not fit.
pub fn offchain_misuse<T: ToTokens>(spanned: T, fn_name: &str, why: OffchainMisuse) -> syn::Error {
    let reason = match why {
        OffchainMisuse::InTraitImpl => {
            "a trait impl cannot have offchain functions: the trait is also implemented by the \
             ContractRef, where an offchain function has no meaning. Move it to a plain `impl` \
             block of the module"
        }
        OffchainMisuse::InFactory => "offchain functions are not supported in a factory module",
        OffchainMisuse::MutReceiver => {
            "an offchain function runs on the host and cannot change the state: take `&self`"
        }
        OffchainMisuse::PayableOrNonReentrant => {
            "an offchain function is never called on chain, so `payable` and `non_reentrant` do \
             not apply to it"
        }
        OffchainMisuse::NoReceiver => {
            "an offchain function reads the contract's state: take `&self`"
        }
    };
    syn::Error::new_spanned(
        spanned,
        format!("`#[odra(offchain)]` on `{fn_name}`: {reason}")
    )
}

// --- Entry points --------------------------------------------------------------------------------

/// An entry point named like a function of the generated refs or factories.
pub fn reserved_fn_name(name: &syn::Ident) -> syn::Error {
    syn::Error::new_spanned(
        name,
        format!("`{name}` is a reserved entry point name; rename the function")
    )
}

/// An entry point argument named like a parameter every call carries.
pub fn reserved_arg_name(name: &syn::Ident) -> syn::Error {
    syn::Error::new_spanned(
        name,
        format!("`{name}` is a reserved argument name; rename the argument")
    )
}

/// An entry point argument that is not `name: Type`, such as a pattern or `self` where a typed
/// argument is expected.
pub fn arg_without_name(arg: &syn::FnArg) -> syn::Error {
    let (spanned, found) = match arg {
        syn::FnArg::Typed(pat_type) => (
            pat_type.pat.to_token_stream(),
            pat_type.pat.to_token_stream().to_string()
        ),
        syn::FnArg::Receiver(receiver) => (receiver.to_token_stream(), "self".to_string())
    };
    syn::Error::new_spanned(
        spanned,
        format!(
            "`{found}` is not an argument name; an entry point argument is written `name: Type`"
        )
    )
}

// --- Structs and enums ---------------------------------------------------------------------------

/// A module struct with unnamed fields: `struct Contract(Var<u32>);`.
pub fn module_fields_unnamed(fields: &syn::Fields) -> syn::Error {
    syn::Error::new_spanned(
        fields,
        "a module's fields are named, as a field's name is its storage key; write \
         `struct Contract { value: Var<u32> }`"
    )
}

/// A module struct with more fields than the storage index holds.
pub fn module_too_many_fields(fields: &syn::Fields, max: usize) -> syn::Error {
    syn::Error::new_spanned(
        fields,
        format!(
            "a module has at most {max} fields, this one has {}; group some in a submodule",
            fields.len()
        )
    )
}

/// An `#[odra::odra_type]` tuple struct: `struct Point(u32, u32);`.
pub fn type_fields_unnamed(fields: &syn::Fields) -> syn::Error {
    syn::Error::new_spanned(
        fields,
        "an Odra type's fields are named, as the name is part of its schema; write \
         `struct Point { x: u32, y: u32 }`"
    )
}

/// `#[odra::event]` on something that is not a struct.
pub fn event_on_other_item(item: &syn::Item) -> syn::Error {
    syn::Error::new(
        item_ident_span(item),
        format!(
            "`#[odra::event]` goes on a struct with named fields, one per event field; this is {}",
            item_kind(item)
        )
    )
}

/// `#[odra::event]` on a tuple or unit struct.
pub fn event_fields_unnamed(fields: &syn::Fields) -> syn::Error {
    syn::Error::new_spanned(
        fields,
        "an event's fields are named, as the name is part of its schema; write \
         `struct Transfer { from: Address, to: Address }`"
    )
}

/// An enum discriminant that is not a `u16` literal: `A = -1`, `A = 70000`, `A = 1 + 1`.
pub fn discriminant_not_u16(discriminant: &syn::Expr) -> syn::Error {
    syn::Error::new_spanned(
        discriminant,
        "a discriminant is an integer literal from 0 to 65535, as an error code is a `u16`"
    )
}

/// An `impl` whose self type is not a plain path, such as `impl (A, B)` or `impl &X`.
pub fn impl_self_type_not_a_path(self_ty: &syn::Type) -> syn::Error {
    syn::Error::new_spanned(
        self_ty,
        "an Odra impl block is for a named module: `impl Contract { .. }`"
    )
}

// --- Spans -------------------------------------------------------------------------------------

/// The span of the item's name when the tokens parse as an item, so the error points at
/// `enum Contract` rather than underlining the whole body; else the span of all the tokens.
fn item_span(item: &TokenStream) -> Span {
    match syn::parse2::<syn::Item>(item.clone()) {
        Ok(parsed) => item_ident_span(&parsed),
        Err(_) => item.span()
    }
}

/// "an enum", "a function", ... for a message that says what the macro was put on.
fn item_kind(item: &syn::Item) -> &'static str {
    match item {
        syn::Item::Const(_) => "a const",
        syn::Item::Enum(_) => "an enum",
        syn::Item::ExternCrate(_) => "an extern crate",
        syn::Item::Fn(_) => "a function",
        syn::Item::ForeignMod(_) => "an extern block",
        syn::Item::Impl(_) => "an impl block",
        syn::Item::Macro(_) => "a macro call",
        syn::Item::Mod(_) => "a module",
        syn::Item::Static(_) => "a static",
        syn::Item::Struct(_) => "a struct",
        syn::Item::Trait(_) => "a trait",
        syn::Item::TraitAlias(_) => "a trait alias",
        syn::Item::Type(_) => "a type alias",
        syn::Item::Union(_) => "a union",
        syn::Item::Use(_) => "a use declaration",
        _ => "something else"
    }
}

fn tokens_kind(item: &TokenStream) -> &'static str {
    match syn::parse2::<syn::Item>(item.clone()) {
        Ok(parsed) => item_kind(&parsed),
        Err(_) => "not an item"
    }
}

fn item_ident_span(item: &syn::Item) -> Span {
    match item {
        syn::Item::Const(i) => i.ident.span(),
        syn::Item::Enum(i) => i.ident.span(),
        syn::Item::Fn(i) => i.sig.ident.span(),
        syn::Item::Mod(i) => i.ident.span(),
        syn::Item::Static(i) => i.ident.span(),
        syn::Item::Struct(i) => i.ident.span(),
        syn::Item::Trait(i) => i.ident.span(),
        syn::Item::Type(i) => i.ident.span(),
        syn::Item::Union(i) => i.ident.span(),
        syn::Item::Impl(i) => i.self_ty.span(),
        other => other.span()
    }
}
