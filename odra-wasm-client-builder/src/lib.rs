#![feature(box_patterns)]

use odra_schema::casper_contract_schema::ContractSchema;
use quote::ToTokens;
use std::{fs::File, io::Write, path::Path, process::Command};

use crate::codegen::{client, types_def};

mod codegen;
mod schema;
mod types;

pub fn generate_wasm_client_code<P: AsRef<Path>>(schema_path: P, working_directory: P) -> Result<(), String> {
    let code = code(schema_path)?;
    write_code(&working_directory, code)?;
    fmt_code(&working_directory)?;
    build_code(&working_directory)?;
    Ok(())
}

fn write_code<P: AsRef<Path>>(path: &P, code: proc_macro2::TokenStream) -> Result<(), String> {
    let path = path.as_ref().join("src/lib.rs");
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    file.write_all(code.to_token_stream().to_string().as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

fn fmt_code<P: AsRef<Path>>(path: &P) -> Result<(), String> {
    Command::new("cargo")
        .arg("fmt")
        .current_dir(path)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn build_code<P: AsRef<Path>>(path: &P) -> Result<(), String> {
    Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("pkg-web")
        .arg("--release")
        .arg(path.as_ref().to_str().ok_or("Invalid path")?)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn read_schema<P: AsRef<Path>>(schema_path: P) -> Result<ContractSchema, String> {
    let schema = std::fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file: {}", e))?;
    let result = serde_json::from_str::<serde_json::Value>(&schema)
        .map_err(|e| format!("Failed to convert schema to value: {}", e))?;

    schema::parse(&result)
}

fn code<P: AsRef<Path>>(schema_path: P) -> Result<proc_macro2::TokenStream, String> {
    let imports = codegen::imports();

    let path = schema_path.as_ref();
    if path.is_dir() {
        // generate code for all schemas in the directory
        let mut clients = proc_macro2::TokenStream::new();
        let mut types = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().map(|s| s == "json").unwrap_or(false) {
                let contract_schema = read_schema(entry.path())?;
                let client = client(&contract_schema);
                types.extend(contract_schema.types);
                clients.extend(client);
            }
        }
        let mut seen = std::collections::HashSet::new();
        let mut unique_types = Vec::new();
        for t in types {
            if seen.contains(&t.name()) {
                continue;
            }
            seen.insert(t.name());
            unique_types.push(t);
        }
        let types = types_def(&unique_types);

        Ok(quote::quote! {
            #imports

            #clients
            #(#types)*
        })
    } else {
        let contract_schema = read_schema(schema_path)?;
        let client = client(&contract_schema);
        let types = types_def(&contract_schema.types);

        Ok(quote::quote! {
            #imports

            #client
            #(#types)*
        })
    }
}