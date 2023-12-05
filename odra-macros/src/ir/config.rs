use proc_macro2::TokenStream;
use quote::ToTokens;
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
    syn::custom_keyword!(events);
}

#[derive(Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents
}

impl Parse for ModuleConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut events = None;

        while !input.is_empty() {
            if events.is_none() && input.peek(kw::events) {
                events = Some(input.parse::<ModuleEvents>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }
            return Err(input.error("Unexpected token"));
        }

        Ok(Self {
            events: events.unwrap_or_default()
        })
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleEvents(Punctuated<ModuleEvent, Token![,]>);

impl ModuleEvents {
    pub fn iter(&self) -> impl Iterator<Item = &ModuleEvent> {
        self.0.iter()
    }
}

impl Parse for ModuleEvents {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // a sample input: events = [Event1, Event2, Event3]
        if input.is_empty() {
            return Ok(Self::default());
        }
        input.parse::<kw::events>()?;
        input.parse::<Token![=]>()?;

        let content: ParseBuffer;
        let _brace_token = syn::bracketed!(content in input);
        let events = Punctuated::parse_terminated(&content)?;
        Ok(Self(events))
    }
}

#[derive(Clone, Debug)]
pub struct ModuleEvent {
    pub ty: syn::Type
}

impl Parse for ModuleEvent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<syn::Type>()?;
        Ok(ModuleEvent { ty })
    }
}

impl ToTokens for ModuleEvent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens);
    }
}
