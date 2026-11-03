use std::ops::Deref;

use super::errors;
use syn::{
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    Token
};

pub enum ConfigItem {
    Module(Box<ModuleConfiguration>),
    Empty
}

impl Parse for ConfigItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Empty);
        }
        let module = input.parse::<ModuleConfiguration>()?;
        Ok(Self::Module(Box::new(module)))
    }
}

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(version);
    syn::custom_keyword!(events);
    syn::custom_keyword!(errors);
    syn::custom_keyword!(factory);
    syn::custom_keyword!(layout);
}

#[derive(Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents,
    pub errors: ModuleErrors,
    pub name: ModuleName,
    pub version: ModuleVersion,
    pub factory: Factory,
    pub layout: ModuleLayout,
    /// Names and spans of the arguments that were actually written, in source order.
    given: Vec<(&'static str, proc_macro2::Span)>
}

impl ModuleConfiguration {
    /// The arguments `#[odra::module(..)]` takes, for error messages.
    const ARGS: [&'static str; 6] = ["name", "version", "events", "errors", "factory", "layout"];

    /// Arguments that only make sense on the module struct, not on an `impl` or `trait` block.
    pub fn struct_only_args(&self) -> impl Iterator<Item = &(&'static str, proc_macro2::Span)> {
        self.given.iter().filter(|(name, _)| *name != "factory")
    }

    /// All arguments that were written.
    pub fn args(&self) -> impl Iterator<Item = &(&'static str, proc_macro2::Span)> {
        self.given.iter()
    }
}

impl Parse for ModuleConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
            match arg {
                "name" => config.name = input.parse()?,
                "version" => config.version = input.parse()?,
                "events" => config.events = input.parse()?,
                "errors" => config.errors = input.parse()?,
                "factory" => config.factory = input.parse()?,
                "layout" => config.layout = input.parse()?,
                _ => unreachable!("`{arg}` comes from `ARGS`")
            }
            let _ = input.parse::<Token![,]>(); // optional comma
        }
        Ok(config)
    }
}

/// An explicit storage layout of the module, overriding the one derived from its fields.
///
/// Used by modules that store data outside of the Odra storage layout, e.g. under named keys:
/// `#[odra::module(layout = odra::schema::named_key::<u8>("decimals"))]`.
#[derive(Default, Clone, Debug)]
pub struct ModuleLayout(Option<syn::Expr>);

impl Deref for ModuleLayout {
    type Target = Option<syn::Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ModuleLayout {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::layout>()?;
        input.parse::<Token![=]>()?;
        let layout = input.parse::<syn::Expr>()?;
        Ok(Self(Some(layout)))
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleName(String);

impl Deref for ModuleName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ModuleName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::name>()?;
        input.parse::<Token![=]>()?;
        let name = input.parse::<syn::LitStr>()?.value();
        Ok(Self(name))
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleVersion(String);

impl Deref for ModuleVersion {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ModuleVersion {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::version>()?;
        input.parse::<Token![=]>()?;
        let version = input.parse::<syn::LitStr>()?.value();
        Ok(Self(version))
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleEvents(Punctuated<ModuleEvent, Token![,]>);

pub type ModuleEvent = syn::Type;

impl ModuleEvents {
    pub fn iter(&self) -> impl Iterator<Item = &ModuleEvent> {
        self.0.iter()
    }
}

impl Parse for ModuleEvents {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // a sample input: events = [Event1, Event2, Event3]
        parse_list::<kw::events>(input).map(Self)
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleErrors(Option<syn::Type>);

impl Deref for ModuleErrors {
    type Target = Option<syn::Type>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ModuleErrors {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ModuleErrors::default());
        }
        input.parse::<kw::errors>()?;
        input.parse::<Token![=]>()?;

        Ok(ModuleErrors(Some(input.parse::<syn::Type>()?)))
    }
}

#[derive(Default, Clone, Debug)]
pub struct Factory(bool);

impl Deref for Factory {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for Factory {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Factory(false));
        }
        input.parse::<kw::factory>()?;
        input.parse::<Token![=]>()?;
        // if 'on' then true, if 'off' then false
        let value = input.parse::<syn::Ident>()?;
        let value = matches!(value.to_string().as_str(), "on");
        Ok(Factory(value))
    }
}

fn parse_list<T: Parse>(
    input: syn::parse::ParseStream
) -> syn::Result<Punctuated<syn::Type, Token![,]>> {
    if input.is_empty() {
        return Ok(Punctuated::default());
    }
    input.parse::<T>()?;
    input.parse::<Token![=]>()?;

    let content: ParseBuffer;
    let _brace_token = syn::bracketed!(content in input);
    Punctuated::parse_terminated(&content)
}
