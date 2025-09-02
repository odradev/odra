use quote::ToTokens;
use std::{fs::File, io::Write, process::Command};

fn main() -> Result<(), String> {
    // Read the path to a schema file
    let schema_path = std::env::args().nth(1).expect("No schema path provided");
    let contract_schema = odra_wasm_client_builder::read_schema(schema_path)?;
    let client = odra_wasm_client_builder::codegen::client(&contract_schema);
    let types = odra_wasm_client_builder::codegen::types_def(&contract_schema);

    let code = quote::quote! {
        #client
        #(#types)*
    };

    write_code(code)?;
    fmt_code()?;
    build_code()?;

    Ok(())
}

fn write_code(code: proc_macro2::TokenStream) -> Result<(), String> {
    let mut file = File::create("../wasm_client/src/lib.rs").map_err(|e| e.to_string())?;
    file.write_all(code.to_token_stream().to_string().as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn fmt_code() -> Result<(), String> {
    Command::new("cargo")
        .arg("fmt")
        .current_dir("../wasm_client")
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn build_code() -> Result<(), String> {
    Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("pkg-web")
        .arg("../wasm_client")
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
