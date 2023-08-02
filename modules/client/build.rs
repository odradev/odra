fn main() {
    // concatenate all files from ../resources into a single file and write it to ../resources/contracts.json
    let mut contracts = Vec::new();
    let files = std::fs::read_dir("../resources").unwrap();
    for file in files {
        let file = file.unwrap();
        let path = file.path();
        if path.is_file() && path.file_name().unwrap() != "contracts.json" {
            let content = std::fs::read_to_string(path).unwrap();
            // load content of the file as json
            let json: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
            contracts.push(json);
        }
    }
    let contracts = serde_json::Value::from(contracts);
    std::fs::write("contracts.json", contracts.to_string()).unwrap();
}