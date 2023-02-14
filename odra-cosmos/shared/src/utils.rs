use odra_cosmos_types::CallArgs;

pub fn build_wasm_message(entry_point: &str, args: CallArgs) -> Vec<u8> {
    let args = args
        .arg_names()
        .iter()
        .map(|name| args.get_as_json(name))
        .collect::<Vec<String>>();
    let args = args.join(",");

    let msg = format!("{{\"args\":[{}],\"name\":\"{}\"}}", args, entry_point);
    msg.as_bytes().to_vec()
}
