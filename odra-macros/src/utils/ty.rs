use quote::ToTokens;
use syn::parse_quote;

pub fn address() -> syn::Type {
    parse_quote!(odra::Address)
}

pub fn address_ref() -> syn::Type {
    parse_quote!(&odra::Address)
}

pub fn contract_env() -> syn::Type {
    parse_quote!(odra::ContractEnv)
}

pub fn rc_contract_env() -> syn::Type {
    parse_quote!(odra::prelude::Rc<odra::ContractEnv>)
}

pub fn from_bytes() -> syn::Type {
    parse_quote!(odra::casper_types::bytesrepr::FromBytes)
}

pub fn to_bytes() -> syn::Type {
    parse_quote!(odra::casper_types::bytesrepr::ToBytes)
}

pub fn bytes_err() -> syn::Type {
    parse_quote!(odra::casper_types::bytesrepr::Error)
}

pub fn event_instance() -> syn::Type {
    parse_quote!(odra::casper_event_standard::EventInstance)
}

pub fn event_error() -> syn::Type {
    parse_quote!(odra::EventError)
}

pub fn u512() -> syn::Type {
    parse_quote!(odra::casper_types::U512)
}

pub fn host_env() -> syn::Type {
    parse_quote!(odra::HostEnv)
}

pub fn call_def() -> syn::Type {
    parse_quote!(odra::CallDef)
}

pub fn entry_points_caller() -> syn::Type {
    parse_quote!(odra::entry_point_callback::EntryPointsCaller)
}

pub fn contract_call_result() -> syn::Type {
    parse_quote!(odra::ContractCallResult)
}

pub fn odra_error() -> syn::Type {
    parse_quote!(odra::OdraError)
}

pub fn odra_result(ty: syn::Type) -> syn::Type {
    parse_quote!(odra::OdraResult<#ty>)
}

pub fn module() -> syn::Type {
    parse_quote!(odra::module::Module)
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
pub fn schema() -> syn::Type {
    parse_quote!(odra::casper_event_standard::Schema)
}

pub fn cl_typed() -> syn::Type {
    parse_quote!(odra::casper_types::CLTyped)
}

pub fn cl_type() -> syn::Type {
    parse_quote!(odra::casper_types::CLType)
}

pub fn cl_type_any() -> syn::Type {
    parse_quote!(odra::casper_types::CLType::Any)
}

pub fn cl_type_u32() -> syn::Type {
    parse_quote!(odra::casper_types::CLType::U32)
}

pub fn runtime_args() -> syn::Type {
    parse_quote!(odra::casper_types::RuntimeArgs)
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
    parse_quote!(odra::prelude::string::String)
}

pub fn result(ty: &syn::Type, err_ty: &syn::Type) -> syn::Type {
    parse_quote!(Result<#ty, #err_ty>)
}

pub fn bytes_result(ty: &syn::Type) -> syn::Type {
    result(ty, &bytes_err())
}

pub fn self_ref() -> syn::Type {
    parse_quote!(&self)
}

pub fn bytes_slice() -> syn::Type {
    parse_quote!(&[u8])
}

#[allow(non_snake_case)]
pub fn _Self() -> syn::Type {
    parse_quote!(Self)
}

pub fn _self() -> syn::Type {
    parse_quote!(self)
}

pub fn vec() -> syn::Type {
    parse_quote!(odra::prelude::vec::Vec)
}

pub fn vec_of(ty: &syn::Type) -> syn::Type {
    parse_quote!(odra::prelude::vec::Vec<#ty>)
}

pub fn bytes_vec() -> syn::Type {
    parse_quote!(odra::prelude::vec::Vec<u8>)
}

pub fn usize() -> syn::Type {
    parse_quote!(usize)
}

pub fn u32() -> syn::Type {
    parse_quote!(u32)
}

pub fn clone() -> syn::Type {
    parse_quote!(::core::clone::Clone)
}

pub fn from<T: ToTokens>(ty: &T) -> syn::Type {
    parse_quote!(::core::convert::From<#ty>)
}

pub fn typed_btree_map(key: &syn::Type, value: &syn::Type) -> syn::Type {
    parse_quote!(odra::prelude::BTreeMap<#key, #value>)
}

pub fn btree_map() -> syn::Type {
    parse_quote!(odra::prelude::BTreeMap)
}

pub fn module_component() -> syn::Type {
    parse_quote!(odra::module::ModuleComponent)
}

pub fn odra_entry_point() -> syn::Type {
    parse_quote!(odra::entry_point_callback::EntryPoint)
}

pub fn odra_entry_point_arg() -> syn::Type {
    parse_quote!(odra::entry_point_callback::EntryPointArgument)
}

fn slice_to_vec(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Slice(ty) => vec_of(ty.elem.as_ref()),
        _ => ty.clone()
    }
}

pub fn unreferenced_ty(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(syn::TypeReference { elem, .. }) => {
            if matches!(**elem, syn::Type::Reference(_)) {
                unreferenced_ty(elem)
            } else {
                slice_to_vec(elem)
            }
        }
        _ => slice_to_vec(ty)
    }
}
