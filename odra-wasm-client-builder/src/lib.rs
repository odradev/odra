#![feature(box_patterns)]

use odra_schema::casper_contract_schema::ContractSchema;
use proc_macro2::TokenStream;
use std::{
    collections::HashSet,
    fs::{read_dir, read_to_string, DirEntry},
    path::Path
};

use crate::error::Result;

mod cmd;
mod codegen;
mod error;
mod types;

pub use error::Error;

pub fn generate_wasm_client_code<P: AsRef<Path>>(
    schema_path: P,
    working_directory: P
) -> Result<()> {
    let code = code(schema_path)?;
    cmd::write(&working_directory, code)?;
    cmd::fmt(&working_directory)?;
    cmd::build(&working_directory)?;
    Ok(())
}

fn code<P: AsRef<Path>>(schema_path: P) -> Result<TokenStream> {
    let imports = codegen::imports();

    let path = schema_path.as_ref();
    if path.is_dir() {
        // generate code for all schemas in the directory
        let mut clients = TokenStream::new();
        let mut types = HashSet::new();
        for entry in read_dir(path)? {
            let entry = entry?;
            if is_json_file(&entry) {
                let contract_schema = read_schema(entry.path())?;
                let client = codegen::client(&contract_schema);
                let user_errors = codegen::user_errors(&contract_schema.contract_name, &contract_schema.errors);

                types.extend(contract_schema.types);
                clients.extend(client);
                clients.extend(user_errors);
            }
        }
        let types = codegen::types_def(types);

        Ok(quote::quote! {
            #imports

            #clients
            #(#types)*
        })
    } else {
        let contract_schema = read_schema(schema_path)?;
        let client = codegen::client(&contract_schema);
        let types = codegen::types_def(contract_schema.types);

        Ok(quote::quote! {
            #imports

            #client
            #(#types)*
        })
    }
}

fn read_schema<P: AsRef<Path>>(schema_path: P) -> Result<ContractSchema> {
    let json = read_to_string(schema_path)?;
    let schema = serde_json::from_str::<ContractSchema>(&json)?;
    Ok(schema)
}

fn is_json_file(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map(|s| s == "json")
        .unwrap_or(false)
}
