use std::ops::Deref;

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
    syn::custom_keyword!(event_mode);
}

#[derive(Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents,
    pub errors: ModuleErrors,
    pub name: ModuleName,
    pub version: ModuleVersion,
    pub factory: Factory,
    pub event_mode: ModuleEventMode
}

impl Parse for ModuleConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut events = None;
        let mut errors = None;
        let mut factory = None;
        let mut event_mode = None;
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

            if factory.is_none() && input.peek(kw::factory) {
                factory = Some(input.parse::<Factory>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            if event_mode.is_none() && input.peek(kw::event_mode) {
                event_mode = Some(input.parse::<ModuleEventMode>()?);
                let _ = input.parse::<Token![,]>(); // optional comma
                continue;
            }

            return Err(input.error("Unexpected token"));
        }

        Ok(Self {
            name: name.unwrap_or_default(),
            version: version.unwrap_or_default(),
            events: events.unwrap_or_default(),
            errors: errors.unwrap_or_default(),
            factory: factory.unwrap_or_default(),
            event_mode: event_mode.unwrap_or_default()
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

/// Which mechanism(s) a module's `emit_event` calls use, mirroring `core::EventMode`.
///
/// Parsed from `event_mode = ces | native | both` on the `impl` block's `#[odra::module(..)]`
/// attribute (the same attribute that carries `factory = on`), since it is the `impl` block's
/// [ModuleConfiguration] that reaches the codegen deciding how the module's root
/// `ContractEnv` is built and whether the native event topic is registered at install.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleEventMode {
    /// Emit events only via CES. The default.
    #[default]
    Ces,
    /// Emit events only via the Casper native mechanism.
    Native,
    /// Emit every event via both mechanisms.
    Both
}

impl Parse for ModuleEventMode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ModuleEventMode::default());
        }
        input.parse::<kw::event_mode>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<syn::Ident>()?;
        match value.to_string().as_str() {
            "ces" => Ok(ModuleEventMode::Ces),
            "native" => Ok(ModuleEventMode::Native),
            "both" => Ok(ModuleEventMode::Both),
            _ => Err(syn::Error::new_spanned(
                value,
                "expected `ces`, `native` or `both`"
            ))
        }
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
