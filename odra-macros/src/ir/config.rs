use std::ops::Deref;

use syn::{
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    Token
};

pub enum ConfigItem {
    Module(ModuleConfiguration),
    Empty
}

impl Parse for ConfigItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Empty);
        }
        let module = input.parse::<ModuleConfiguration>()?;
        Ok(Self::Module(module))
    }
}

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(version);
    syn::custom_keyword!(events);
    syn::custom_keyword!(errors);
}

#[derive(Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents,
    pub errors: ModuleErrors,
    pub name: ModuleName,
    pub version: ModuleVersion
}

impl Parse for ModuleConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut events = None;
        let mut errors = None;

        while !input.is_empty() {
            if events.is_none() && input.peek(kw::events) {
                events = Some(input.parse::<ModuleEvents>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            if errors.is_none() && input.peek(kw::errors) {
                errors = Some(input.parse::<ModuleErrors>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            if name.is_none() && input.peek(kw::name) {
                name = Some(input.parse::<ModuleName>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            if version.is_none() && input.peek(kw::version) {
                version = Some(input.parse::<ModuleVersion>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }
            return Err(input.error("Unexpected token"));
        }

        Ok(Self {
            name: name.unwrap_or_default(),
            version: version.unwrap_or_default(),
            events: events.unwrap_or_default(),
            errors: errors.unwrap_or_default()
        })
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
