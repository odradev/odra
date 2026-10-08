// The readable fields and their types, formatted for display.
// Pure functions: no reader, no node, no DOM.

/** A NamedCLType as serialized in the layout file: `"U256"`, `"MyStruct"`, `{ "Option": "U8" }`, ... */
export type Ty = string | { [variant: string]: any };

/** A value that can be read, as `StorageReader.readableFields` returns it. */
export interface Field {
  /** The path, as `StorageReader.locate` takes it. */
  path: string;
  /** The types of the keys the path needs, in order. */
  keys: Ty[];
  type: Ty;
  storage:
    | { kind: "value" | "list_item" | "list_length" | "sequence" }
    | { kind: "named_key" | "dictionary"; name: string };
}

/** How a field is stored, e.g. `mapping`, `dictionary "balances"`. */
export function storageLabel(field: Field): string {
  const { storage } = field;
  switch (storage.kind) {
    case "value":
      return field.keys.length ? "mapping" : "value";
    case "named_key":
      return `named key "${storage.name}"`;
    case "dictionary":
      return `dictionary "${storage.name}"`;
    default:
      return storage.kind.replace("_", " ");
  }
}

/** A Rust-like name of a type, e.g. `Option<U8>`, `(Key, Key)`. */
export function typeName(ty: Ty): string {
  if (typeof ty === "string") return ty;
  const [variant, inner] = Object.entries(ty)[0];
  switch (variant) {
    case "Option":
    case "List":
      return `${variant}<${typeName(inner)}>`;
    case "ByteArray":
      return `[u8; ${inner}]`;
    case "Result":
      return `Result<${typeName(inner.ok)}, ${typeName(inner.err)}>`;
    case "Map":
      return `Map<${typeName(inner.key)}, ${typeName(inner.value)}>`;
    case "Tuple1":
    case "Tuple2":
    case "Tuple3":
      return `(${(inner as Ty[]).map(typeName).join(", ")})`;
    case "Custom":
      return inner;
    default:
      return variant;
  }
}

/** The odra-cli text format of a key, see `format_type_hint` in `odra_schema::codec`. */
export function keyHint(ty: Ty): string {
  if (typeof ty === "string") {
    const hints: Record<string, string> = {
      Bool: "true | false",
      I32: "integer",
      I64: "integer",
      U8: "0-255 | 0x00",
      U32: "unsigned integer",
      U64: "unsigned integer",
      U128: "decimal",
      U256: "decimal",
      U512: "decimal",
      String: "text",
      Key: "account-hash-… | hash-…",
      URef: "uref-…-007",
      PublicKey: "public key hex"
    };
    return hints[ty] ?? ty;
  }
  const [variant, inner] = Object.entries(ty)[0];
  switch (variant) {
    case "Option":
      return `none | some:${keyHint(inner)}`;
    case "Tuple1":
      return keyHint(inner[0]);
    case "Tuple2":
    case "Tuple3":
      return (inner as Ty[]).map(keyHint).join(":");
    case "ByteArray":
      return "0x…";
    default:
      return typeName(ty);
  }
}
