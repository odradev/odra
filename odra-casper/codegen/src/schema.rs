use odra_types::{
    casper_types::CLType,
    contract_def::{Argument, Entrypoint, Event}
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
            return_ty: ep.ret.clone()
        }
    }
}

impl From<&Argument> for ElemDef {
    fn from(arg: &Argument) -> Self {
        ElemDef {
            name: arg.ident.clone(),
            ty: arg.ty.clone()
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
