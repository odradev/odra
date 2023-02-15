use serde_json::{json, Map, Value};

/// The input should be a valid json object with the following structure:
///
/// {
///     "name": "entry point to call",
///     "args": [
///         { "arg_name_1": 1 },
///         { "arg_name_2": "string" },
///         { "arg_name_3": 123 },
///     ]
/// }
const ARG_ACTION_NAME: &str = "name";
const ARG_ACTION_ARGS: &str = "args";

fn main() {
    let args = std::env::args().nth(1).expect("argument not found");
    let json: Value = serde_json::from_str(&args).expect("the argument should be a valid json");

    // get the value of field `name` as a string
    let name = json
        .get(ARG_ACTION_NAME)
        .expect("the name argument not found")
        .as_str()
        .expect("the name should be a string");

    let empty = &Value::Array(vec![]);
    // get the value of field `args` as an array, or an empty array if the `args` field is not present.
    let args = json
        .get(ARG_ACTION_ARGS)
        .unwrap_or(empty)
        .as_array()
        .expect("args should be an array");

    // serialize arguments' values to bytes
    let serialized_args = args
        .iter()
        .map(|value| {
            let value = value.as_object().expect("An argument should be an object");
            let value = value
                .values()
                .last()
                .expect("The value should be an object with exactly one property");
            let bytes = serde_json::to_vec(value).expect("Couldn't serialize an argument");
            Value::Array(bytes.iter().map(|v| json!(v)).collect())
        })
        .collect::<Vec<_>>();

    // build back a json object, but with serialized args
    let result = build_message(name, serialized_args);
    println!("{}", result);
}

fn build_message(entry_point: &str, args: Vec<Value>) -> String {
    let args = Value::Array(args);
    let mut ep = Map::new();
    ep.insert(
        ARG_ACTION_NAME.to_string(),
        Value::String(entry_point.to_string()),
    );
    ep.insert(ARG_ACTION_ARGS.to_string(), args);
    let root = Value::Object(ep);
    serde_json::to_string(&root).unwrap()
}
