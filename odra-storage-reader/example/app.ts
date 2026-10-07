// The storage explorer: lists the deployed contracts and reads any field of their storage.
import type * as Casper from "casper-js-sdk";
import init, { StorageReader } from "odra-storage-reader";

// The browser build of casper-js-sdk (loaded in index.html) defines its classes as globals.
const sdk = globalThis as unknown as typeof Casper;
// The node, through the proxy of server.ts.
const rpc = new sdk.RpcClient(new sdk.HttpHandler(`${location.origin}/rpc`));
// The node's answer for a value that was never written.
const QUERY_FAILED = -32003;
// casper-js-sdk throws an `HttpError` with the RPC error as `sourceErr`.
const isNotSet = (e: any) => (e?.sourceErr?.code ?? e?.code) === QUERY_FAILED;

interface Contract {
  name: string;
  packageHash: string;
  hasLayout: boolean;
}

// A NamedCLType as serialized in the layout file: `"U256"`, `"MyStruct"`, `{ "Option": "U8" }`, ...
type Ty = string | { [variant: string]: any };

// A storage node of the layout file (`StorageKind`, with the field name when it is a module field).
interface LayoutNode {
  kind: "module" | "value" | "mapping" | "list" | "sequence" | "named_key" | "dictionary";
  name?: string;
  [property: string]: any;
}

// A readable field: its path, the keys it needs (in order) and the type of the value.
interface Field {
  path: string;
  keys: Ty[];
  ty: Ty;
  storage: string;
}

interface Opened {
  contract: Contract;
  reader: StorageReader;
  fields: Field[];
  contractHash: Promise<string>;
}

const app = document.getElementById("app")!;
const opened = new Map<string, Opened>();
let contracts: Promise<Contract[]>;

// ---------------------------------------------------------------------------------------------
// Layout

function collectFields(node: LayoutNode, path: string, keys: Ty[], out: Field[]): Field[] {
  const join = (name: string) => (path ? `${path}.${name}` : name);
  switch (node.kind) {
    case "module":
      for (const field of node.fields as LayoutNode[]) collectFields(field, join(field.name!), keys, out);
      break;
    case "mapping":
      collectFields(node.value, path, [...keys, node.key], out);
      break;
    case "list":
      out.push({ path, keys: [...keys, "U32"], ty: node.item, storage: "list item" });
      out.push({ path: join("len"), keys, ty: "U32", storage: "list length" });
      break;
    case "sequence":
      out.push({ path, keys, ty: node.ty, storage: "sequence" });
      break;
    case "named_key":
      out.push({ path, keys, ty: node.ty, storage: `named key "${node.key_name}"` });
      break;
    case "dictionary":
      out.push({
        path,
        keys: [...keys, node.key],
        ty: node.value,
        storage: `dictionary "${node.dictionary_name}"`
      });
      break;
    default:
      out.push({ path, keys, ty: node.ty, storage: keys.length ? "mapping" : "value" });
  }
  return out;
}

function typeName(ty: Ty): string {
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

// The odra-cli text format of a key, see `format_type_hint` in odra-cli.
function keyHint(ty: Ty): string {
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

// ---------------------------------------------------------------------------------------------
// Node

async function open(contract: Contract): Promise<Opened> {
  const cached = opened.get(contract.name);
  if (cached) return cached;
  const response = await fetch(`layouts/${contract.name}.json`);
  if (!response.ok) throw new Error(`No layout for ${contract.name}, run \`just export-layouts\``);
  const reader = new StorageReader(await response.text());
  const contractHash = rpc
    .queryLatestGlobalState(contract.packageHash, [])
    .then((pkg) => reader.currentContractHash(pkg.rawJSON.stored_value));
  const entry = { contract, reader, fields: collectFields(reader.fields(), "", [], []), contractHash };
  opened.set(contract.name, entry);
  return entry;
}

async function read(contract: Opened, field: Field, keys: string[], raw: boolean) {
  const location = contract.reader.locate(field.path, keys);
  const contractHash = await contract.contractHash;
  try {
    const result =
      location.kind === "dictionary"
        ? await rpc.getDictionaryItemByIdentifier(
            null,
            new sdk.ParamDictionaryIdentifier(
              undefined,
              new sdk.ParamDictionaryIdentifierContractNamedKey(
                contractHash,
                location.dictionaryName,
                location.dictionaryItemKey
              )
            )
          )
        : await rpc.queryLatestGlobalState(contractHash, [location.name]);
    const value = contract.reader.decode(result.rawJSON.stored_value.CLValue, location, { raw });
    return { location, value: value as string | null };
  } catch (e) {
    if (isNotSet(e)) return { location, value: null };
    throw e;
  }
}

// ---------------------------------------------------------------------------------------------
// View

type Child = Node | string | null | undefined | false;

function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Partial<Record<string, any>> = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (key.startsWith("on")) el.addEventListener(key.slice(2), value);
    else if (key === "class") el.className = value;
    else if (value !== undefined && value !== false) el.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children) {
    if (child) el.append(child);
  }
  return el;
}

const short = (hash: string) => hash.replace(/^(\w+-)?(.{8}).*(.{6})$/, "$1$2…$3");
const message = (e: unknown) => (e instanceof Error ? e.message : String(e));

function show(...children: Child[]) {
  app.replaceChildren(...(children.filter(Boolean) as (Node | string)[]));
}

async function contractsView() {
  show(h("p", { class: "muted" }, "Loading contracts…"));
  let list: Contract[];
  try {
    list = await contracts;
  } catch (e) {
    return show(h("div", { class: "error" }, message(e)));
  }
  show(
    h("header", {}, h("h1", {}, "Deployed contracts"), h("p", { class: "muted" }, `${list.length} contracts`)),
    h(
      "div",
      { class: "grid" },
      ...list.map((contract) =>
        h(
          contract.hasLayout ? "a" : "div",
          {
            class: `card${contract.hasLayout ? "" : " disabled"}`,
            href: contract.hasLayout ? `#/${encodeURIComponent(contract.name)}` : undefined
          },
          h("span", { class: "name" }, contract.name),
          h("code", { title: contract.packageHash }, short(contract.packageHash)),
          contract.hasLayout ? null : h("span", { class: "badge" }, "no layout")
        )
      )
    )
  );
}

async function readerView(name: string) {
  show(h("p", { class: "muted" }, "Loading layout…"));
  let contract: Opened;
  try {
    const found = (await contracts).find((c) => c.name === name);
    if (!found) throw new Error(`Unknown contract ${name}`);
    contract = await open(found);
  } catch (e) {
    return show(backLink(), h("div", { class: "error" }, message(e)));
  }

  const hash = h("code", {}, "resolving…");
  contract.contractHash.then(
    (value) => hash.replaceChildren(value),
    (e) => hash.replaceChildren(h("span", { class: "error-text" }, message(e)))
  );

  const panel = h("section", { class: "panel" }, h("p", { class: "muted" }, "Select a field."));
  const items = contract.fields.map((field) =>
    h(
      "button",
      {
        class: "field",
        onclick: (event: Event) => {
          for (const item of items) item.classList.remove("selected");
          (event.currentTarget as HTMLElement).classList.add("selected");
          panel.replaceChildren(...fieldPanel(contract, field));
        }
      },
      h("span", { class: "path" }, field.path),
      h("span", { class: "type" }, typeName(field.ty)),
      field.keys.length ? h("span", { class: "badge" }, `${field.keys.length} key${field.keys.length > 1 ? "s" : ""}`) : null
    )
  );

  show(
    backLink(),
    h(
      "header",
      {},
      h("h1", {}, contract.contract.name),
      h(
        "dl",
        {},
        h("dt", {}, "Package"),
        h("dd", {}, h("code", {}, contract.contract.packageHash)),
        h("dt", {}, "Contract"),
        h("dd", {}, hash)
      )
    ),
    h("div", { class: "reader" }, h("nav", { class: "fields" }, ...items), panel)
  );
}

function fieldPanel(contract: Opened, field: Field): Node[] {
  const inputs = field.keys.map((ty) => h("input", { placeholder: keyHint(ty), spellcheck: "false" }));
  const raw = h("input", { type: "checkbox" });
  const result = h("div", { class: "result" });

  const submit = async (event: Event) => {
    event.preventDefault();
    result.replaceChildren(h("p", { class: "muted" }, "Reading…"));
    try {
      const { location, value } = await read(
        contract,
        field,
        inputs.map((input) => input.value.trim()),
        raw.checked
      );
      result.replaceChildren(
        h(
          "dl",
          {},
          ...(location.kind === "dictionary"
            ? [
                h("dt", {}, "Dictionary"),
                h("dd", {}, h("code", {}, location.dictionaryName)),
                h("dt", {}, "Item key"),
                h("dd", {}, h("code", {}, location.dictionaryItemKey))
              ]
            : [h("dt", {}, "Named key"), h("dd", {}, h("code", {}, location.name))])
        ),
        value === null
          ? h("p", { class: "value unset" }, "Not set")
          : h("pre", { class: "value" }, raw.checked ? `0x${value}` : value)
      );
    } catch (e) {
      result.replaceChildren(h("div", { class: "error" }, message(e)));
    }
  };

  return [
    h("h2", {}, h("code", {}, field.path)),
    h(
      "p",
      { class: "muted" },
      `${typeName(field.ty)} · ${field.storage}`
    ),
    h(
      "form",
      { onsubmit: submit },
      ...field.keys.map((ty, i) =>
        h("label", {}, h("span", {}, `Key ${i + 1} · ${typeName(ty)}`), inputs[i])
      ),
      h("label", { class: "check" }, raw, h("span", {}, "Raw bytes")),
      h("button", { type: "submit", class: "primary" }, "Read")
    ),
    result
  ];
}

function backLink() {
  return h("a", { href: "#/", class: "back" }, "← Contracts");
}

function route() {
  const name = decodeURIComponent(location.hash.replace(/^#\/?/, ""));
  if (name) readerView(name);
  else contractsView();
}

await init();
contracts = fetch("/api/contracts").then(async (response) => {
  const body = await response.json();
  if (!response.ok) throw new Error(body.error);
  return body as Contract[];
});
window.addEventListener("hashchange", route);
route();
