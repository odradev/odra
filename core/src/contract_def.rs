use odra_types::CLType;

#[derive(Debug)]
pub struct ContractDef {
    pub ident: String,
    pub entrypoints: Vec<Entrypoint>,
}

#[derive(Debug)]
pub struct Entrypoint {
    pub ident: String,
    pub args: Vec<Argument>,
    pub ret: CLType,
}

#[derive(Debug)]
pub struct Argument {
    pub ident: String,
    pub ty: CLType,
}

pub trait HasContractDef {
    fn contract_def() -> ContractDef;
}
