use odra_schema::casper_contract_schema::ContractSchema;

pub fn load_schemas(schemas_str: &str) -> Result<Vec<ContractSchema>, String> {
    let schemas: Vec<ContractSchema> = match serde_json::from_str(schemas_str) {
        Ok(schemas) => schemas,
        Err(_) => return Err("Error parsing contract schemas".to_string())
    };
    Ok(schemas)
}

pub fn assert_contract_exists_in_schema(
    contract_name: &str,
    schemas: Vec<ContractSchema>
) -> Result<(), String> {
    match schemas.iter().find(|s| s.contract_name == contract_name) {
        None => Err(format!(
            "Could not find a {contract_name} contract in schemas."
        )),
        Some(_) => Ok(())
    }
}
