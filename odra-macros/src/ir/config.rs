use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token
};

use super::errors;

/// The arguments of `#[odra::module(..)]`, each given at most once, in any order.
///
/// ```ignore
/// #[odra::module(
///     name = "MyToken",                // the contract's name, the struct's by default
///     version = "1.0.0",               // the contract's version, the crate's by default
///     events = [Transfer, Approval],   // the events the module emits itself
///     errors = TokenError,             // the error enum the module reverts with
///     factory = on,                    // also generate the factory of the module
///     layout = named_key::<u8>("x")    // the storage layout, derived from the fields by default
/// )]
/// ```
#[derive(Default)]
pub struct ModuleConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub events: Vec<syn::Type>,
    pub errors: Option<syn::Type>,
    pub factory: bool,
    pub layout: Option<syn::Expr>,
    /// Names and spans of the arguments that were written, in source order.
    given: Vec<(&'static str, Span)>
}

impl ModuleConfig {
    /// The arguments `#[odra::module(..)]` takes, for error messages.
    const ARGS: [&'static str; 6] = ["name", "version", "events", "errors", "factory", "layout"];

    /// The written arguments that only make sense on the module struct, not on its impl block.
    pub fn struct_only_args(&self) -> impl Iterator<Item = &(&'static str, Span)> {
        self.given.iter().filter(|(name, _)| *name != "factory")
    }

    /// All the arguments that were written.
    pub fn args(&self) -> impl Iterator<Item = &(&'static str, Span)> {
        self.given.iter()
    }
}

impl Parse for ModuleConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = Self::default();
        while !input.is_empty() {
            let span = input.span();
            let ident = input.fork().parse::<syn::Ident>().ok();
            let arg = ident
                .and_then(|ident| Self::ARGS.into_iter().find(|arg| ident == arg))
                .ok_or_else(|| errors::unknown_module_arg(input, &Self::ARGS))?;
            if config.given.iter().any(|(given, _)| *given == arg) {
                return Err(errors::module_arg_twice(arg, span));
            }
            config.given.push((arg, span));
            input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;
            match arg {
                "name" => config.name = Some(input.parse::<syn::LitStr>()?.value()),
                "version" => config.version = Some(input.parse::<syn::LitStr>()?.value()),
                "events" => {
                    let content;
                    syn::bracketed!(content in input);
                    let events = Punctuated::<syn::Type, Token![,]>::parse_terminated(&content)?;
                    config.events = events.into_iter().collect();
                }
                "errors" => config.errors = Some(input.parse()?),
                "factory" => config.factory = parse_on_off(input)?,
                "layout" => config.layout = Some(input.parse()?),
                _ => unreachable!("`{arg}` comes from `ARGS`")
            }
            let _ = input.parse::<Token![,]>(); // optional comma
        }
        Ok(config)
    }
}

/// `on` or `off`.
fn parse_on_off(input: ParseStream) -> syn::Result<bool> {
    let value = input.parse::<syn::Ident>()?;
    match value.to_string().as_str() {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => Err(errors::factory_not_on_or_off(&value))
    }
}
