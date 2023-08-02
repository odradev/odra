use odra_casper_codegen::schema::Schema;

pub fn load_schemas(schemas_str: &str) -> Result<Vec<Schema>, String> {
    let schemas: Vec<Schema> = match serde_json::from_str(schemas_str) {
        Ok(schemas) => schemas,
        Err(_) => return Err("Error parsing contract schemas".to_string()),
    };
    Ok(schemas)
}

pub fn assert_contract_exists_in_schema(contract_name: &str, schemas: Vec<Schema>) -> Result<(), String>{
    match schemas.iter().find(|s| s.name == contract_name) {
        None => { Err(format!("Could not find a {contract_name} contract in schemas.")) }
        Some(_) => { Ok(())}
    }
}
