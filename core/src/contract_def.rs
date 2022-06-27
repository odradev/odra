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
    pub ty: EntrypointType,
}

#[derive(Debug)]
pub struct Argument {
    pub ident: String,
    pub ty: CLType,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntrypointType {
    Constructor,
    Public,
}

pub trait HasContractDef {
    fn contract_def() -> ContractDef;
}
