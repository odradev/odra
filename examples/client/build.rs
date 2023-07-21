use std::collections::HashMap;

fn main() {
    // concatenate all files from ../resources into a single file and write it to ../resources/contracts.json
    let mut contracts = Vec::new();
    let files = std::fs::read_dir("../resources").unwrap();
    for file in files {
        let file = file.unwrap();
        let path = file.path();
        if path.is_file() {
            let content = std::fs::read_to_string(path).unwrap();
            // load content of the file as json
            let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
            contracts.push(json);
        }
    }
    let contracts = serde_json::Value::from(contracts);
    std::fs::write("contracts.json", contracts.to_string()).unwrap();

    // load all contents of files in ../wasm folder as bytes into a vector and write it to ../resources/wasm.json
    let mut wasm: HashMap<String, Vec<u8>> = HashMap::new();
    let files = std::fs::read_dir("../wasm").unwrap();
    for file in files {
        let wasm_file = file.unwrap();
        let path = wasm_file.path();
        if path.is_file() {
            let content = std::fs::read(path).unwrap();
            wasm.insert(wasm_file.file_name().to_str().unwrap().to_string(), content);
        }
    }

    let serialized = bincode::serialize(&wasm).unwrap();

    // write the serialized vector to ../resources/wasm.json
    std::fs::write("contracts.bin", serialized).unwrap();
}