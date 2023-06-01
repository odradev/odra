use casper_types::CLType;
use odra_types::{
    contract_def::{Argument, Entrypoint, Event},
    Type
};
use serde::{Deserialize, Serialize};

pub fn gen_schema(
    contract_ident: &str,
    contract_entrypoints: &[Entrypoint],
    events: &[Event]
) -> String {
    let schema = Schema {
        name: contract_ident.to_owned(),
        entrypoints: contract_entrypoints.iter().map(Into::into).collect(),
        events: events.iter().map(Into::into).collect()
    };
    serde_json::to_string_pretty(&schema).expect("Failed to serialize schema")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Schema {
    name: String,
    entrypoints: Vec<EntrypointDef>,
    events: Vec<EventDef>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntrypointDef {
    name: String,
    is_mutable: bool,
    args: Vec<ElemDef>,
    return_ty: CLType
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventDef {
    name: String,
    fields: Vec<ElemDef>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ElemDef {
    name: String,
    ty: CLType
}

impl From<&Entrypoint> for EntrypointDef {
    fn from(ep: &Entrypoint) -> Self {
        EntrypointDef {
            name: ep.ident.clone(),
            is_mutable: ep.is_mut,
            args: ep.args.iter().map(Into::into).collect(),
            return_ty: type_to_cl_type(&ep.ret)
        }
    }
}

impl From<&Argument> for ElemDef {
    fn from(arg: &Argument) -> Self {
        ElemDef {
            name: arg.ident.clone(),
            ty: type_to_cl_type(&arg.ty)
        }
    }
}

impl From<&Event> for EventDef {
    fn from(event: &Event) -> Self {
        EventDef {
            name: event.ident.clone(),
            fields: event.args.iter().map(Into::into).collect()
        }
    }
}

fn type_to_cl_type(ty: &Type) -> CLType {
    match ty {
        Type::Address => CLType::Key,
        Type::Bool => CLType::Bool,
        Type::I32 => CLType::I32,
        Type::I64 => CLType::I64,
        Type::U8 => CLType::U8,
        Type::U32 => CLType::U32,
        Type::U64 => CLType::U64,
        Type::U128 => CLType::U128,
        Type::U256 => CLType::U256,
        Type::U512 => CLType::U512,
        Type::Unit => CLType::Unit,
        Type::String => CLType::String,
        Type::Option(v) => CLType::Option(Box::new(type_to_cl_type(v))),
        Type::Result { ok, err } => CLType::Result {
            ok: Box::new(type_to_cl_type(ok)),
            err: Box::new(type_to_cl_type(err))
        },
        Type::Map { key, value } => CLType::Map {
            key: Box::new(type_to_cl_type(key)),
            value: Box::new(type_to_cl_type(value))
        },
        Type::Tuple1(v) => CLType::Tuple1(v.clone().map(|v| Box::new(type_to_cl_type(&v)))),
        Type::Tuple2(v) => CLType::Tuple2(v.clone().map(|v| Box::new(type_to_cl_type(&v)))),
        Type::Tuple3(v) => CLType::Tuple3(v.clone().map(|v| Box::new(type_to_cl_type(&v)))),
        Type::Any => CLType::Any,
        Type::Vec(v) => CLType::List(Box::new(type_to_cl_type(v))),
        Type::ByteArray(v) => CLType::ByteArray(*v),
        Type::Slice(ty) => CLType::List(Box::new(type_to_cl_type(ty)))
    }
}
