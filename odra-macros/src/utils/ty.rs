use syn::parse_quote;

pub fn address() -> syn::Type {
    parse_quote!(odra::Address)
}

pub fn contract_env() -> syn::Type {
    parse_quote!(odra::ContractEnv)
}

pub fn from_bytes() -> syn::Type {
    parse_quote!(odra::FromBytes)
}

pub fn event_instance() -> syn::Type {
    parse_quote!(odra::casper_event_standard::EventInstance)
}

pub fn event_error() -> syn::Type {
    parse_quote!(odra::event::EventError)
}

pub fn u512() -> syn::Type {
    parse_quote!(odra::U512)
}

pub fn host_env() -> syn::Type {
    parse_quote!(odra::HostEnv)
}

pub fn call_def() -> syn::Type {
    parse_quote!(odra::CallDef)
}

pub fn entry_points_caller() -> syn::Type {
    parse_quote!(odra::EntryPointsCaller)
}

pub fn contract_call_result() -> syn::Type {
    parse_quote!(odra::ContractCallResult)
}

pub fn odra_error() -> syn::Type {
    parse_quote!(odra::OdraError)
}

pub fn module_wrapper() -> syn::Type {
    parse_quote!(odra::ModuleWrapper)
}

pub fn module() -> syn::Type {
    parse_quote!(odra::Module)
}

pub fn variable() -> syn::Type {
    parse_quote!(odra::Variable)
}

pub fn mapping() -> syn::Type {
    parse_quote!(odra::Mapping)
}

pub fn entry_points() -> syn::Type {
    parse_quote!(odra::casper_types::EntryPoints)
}

pub fn entry_point() -> syn::Type {
    parse_quote!(odra::casper_types::EntryPoint)
}

pub fn entry_point_access() -> syn::Type {
    parse_quote!(odra::casper_types::EntryPointAccess)
}

pub fn entry_point_type() -> syn::Type {
    parse_quote!(odra::casper_types::EntryPointType)
}

pub fn parameter() -> syn::Type {
    parse_quote!(odra::casper_types::Parameter)
}

pub fn group() -> syn::Type {
    parse_quote!(odra::casper_types::Group)
}

pub fn schemas() -> syn::Type {
    parse_quote!(odra::casper_event_standard::Schemas)
}

pub fn cl_typed() -> syn::Type {
    parse_quote!(odra::casper_types::CLTyped)
}

pub fn runtime_args() -> syn::Type {
    parse_quote!(odra::RuntimeArgs)
}

pub fn has_events() -> syn::Type {
    parse_quote!(odra::contract_def::HasEvents)
}

pub fn has_entrypoints() -> syn::Type {
    parse_quote!(odra::contract_def::HasEntrypoints)
}

pub fn has_ident() -> syn::Type {
    parse_quote!(odra::contract_def::HasIdent)
}

pub fn event() -> syn::Type {
    parse_quote!(odra::contract_def::Event)
}

pub fn contract_blueprint() -> syn::Type {
    parse_quote!(odra::contract_def::ContractBlueprint)
}

pub fn entry_point_def() -> syn::Type {
    parse_quote!(odra::contract_def::Entrypoint)
}

pub fn entry_point_def_attr_payable() -> syn::Type {
    parse_quote!(odra::contract_def::EntrypointAttribute::Payable)
}

pub fn entry_point_def_attr_non_reentrant() -> syn::Type {
    parse_quote!(odra::contract_def::EntrypointAttribute::NonReentrant)
}

pub fn entry_point_def_ty_constructor() -> syn::Type {
    parse_quote!(odra::contract_def::EntrypointType::Constructor)
}

pub fn entry_point_def_ty_public() -> syn::Type {
    parse_quote!(odra::contract_def::EntrypointType::Public)
}

pub fn entry_point_def_arg() -> syn::Type {
    parse_quote!(odra::contract_def::Argument)
}

pub fn string() -> syn::Type {
    parse_quote!(String)
}
