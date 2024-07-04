use odra_casper_js_client::schemas::Contracts;
use serde_json::value::Value;
use std::collections::BTreeMap;

fn main() {
    // concatenate all files from ../resources into a single file and write it to ../resources/contracts.json
    let mut contracts = Vec::new();
    let files = std::fs::read_dir("../resources/casper_contract_schemas").unwrap();
    for file in files {
        let file = file.unwrap();
        let path = file.path();
        if path.is_file() && path.file_name().unwrap() != "contracts.json" {
            let content = std::fs::read_to_string(path).unwrap();
            // load content of the file as json
            let schema: Value = serde_json::from_str(content.as_str()).unwrap();
            contracts.push(schema);
        }
    }

    let mut wasm_bytes = BTreeMap::default();
    for contract in &contracts {
        let name = contract["contract_name"].as_str().unwrap();
        let contract_bytes = std::fs::read(format!("../wasm/{}.wasm", name)).unwrap();

        wasm_bytes.insert(name.to_string(), contract_bytes);
    }

    let contracts = Contracts::new(contracts, wasm_bytes);
    std::fs::write("contracts.json", serde_json::to_string(&contracts).unwrap()).unwrap();
}
