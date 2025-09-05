use odra_schema::casper_contract_schema::{
    Access, Argument, ContractSchema, CustomType, Entrypoint, NamedCLType, Type
};
use serde_json::Value;

fn parse_custom_types(v: &Value) -> Result<Vec<CustomType>, String> {
    v["types"]
        .as_array()
        .ok_or_else(|| "Expected types to be an array".to_string())?
        .iter()
        .cloned()
        .map(|v| {
            serde_json::from_value::<CustomType>(v.clone())
                .map_err(|e| format!("Failed to parse type: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn parse_entry_points(v: &Value, types: &[CustomType]) -> Result<Vec<Entrypoint>, String> {
    v["entry_points"]
        .as_array()
        .ok_or_else(|| "Expected entry_points to be an array".to_string())?
        .iter()
        .cloned()
        .map(|v| {
            Ok(Entrypoint {
                name: v["name"].as_str().unwrap_or_default().into(),
                description: v["description"].as_str().map(String::from),
                is_mutable: v["is_mutable"].as_bool().unwrap_or_default(),
                arguments: v["arguments"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(|arg| {
                        Ok(Argument {
                            name: arg["name"].as_str().unwrap_or_default().into(),
                            description: None,
                            ty: parse_type(&arg["ty"], types)?,
                            optional: arg["optional"].as_bool().unwrap_or_default()
                        })
                    })
                    .collect::<Result<Vec<Argument>, String>>()?,
                return_ty: parse_type(&v["return_ty"], types)?,
                is_contract_context: v["is_contract_context"].as_bool().unwrap_or_default(),
                access: serde_json::from_value::<Access>(v["access"].clone())
                    .map_err(|e| format!("Failed to parse access: {}", e))?
            })
        })
        .collect::<Result<_, _>>()
}

fn parse_type(value: &Value, _custom_types: &[CustomType]) -> Result<Type, String> {
    let res = serde_json::from_value::<Type>(value.clone());
    match res {
        Ok(ty) => Ok(ty),
        Err(_) => match value.clone() {
            Value::Object(ref map) => {
                if map.contains_key("Option") {
                    let inner_value = map.get("Option").unwrap();
                    let it = parse_type(inner_value, _custom_types)?;
                    Ok(Type(NamedCLType::Option(Box::new(it.0))))
                } else {
                    Err("Unsupported empty object type".to_string())
                }
            }
            Value::String(ty_name) => {
                Ok(Type(NamedCLType::Custom(ty_name.to_string())))
            }
            _ => Err(format!("Unsupported type format {:?}", value))
        }
    }
}

pub fn parse(v: &Value) -> Result<ContractSchema, String> {
    let types = parse_custom_types(v)?;
    let entry_points = parse_entry_points(v, &types)?;
    Ok(ContractSchema {
        casper_contract_schema_version: v["casper_contract_schema_version"].as_u64().unwrap_or(1)
            as u8,
        toolchain: v["toolchain"].as_str().unwrap_or_default().to_string(),
        authors: v["authors"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        repository: v["repository"].as_str().map(String::from),
        homepage: v["homepage"].as_str().map(String::from),
        contract_name: v["contract_name"].as_str().unwrap_or_default().to_string(),
        contract_version: v["contract_version"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        types,
        errors: vec![],
        entry_points,
        events: vec![],
        call: None
    })
}
