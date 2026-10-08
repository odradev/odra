// The storage explorer page: lists the deployed contracts and reads any field of their storage.
// Presentation only - see storage.ts for how the storage is read.
import { keyHint, storageLabel, typeName, type Field } from "./layout.js";
import {
  ContractStorage,
  fetchContracts,
  loadReader,
  type Contract,
  type ReadResult
} from "./storage.js";

const app = document.getElementById("app")!;
let contracts: Promise<Contract[]>;

// ---------------------------------------------------------------------------------------------
// Views

async function contractsView() {
  show(h("p", { class: "muted" }, "Loading contracts…"));
  let list: Contract[];
  try {
    list = await contracts;
  } catch (e) {
    return show(errorBox(e));
  }
  show(
    h("header", {}, h("h1", {}, "Deployed contracts"), h("p", { class: "muted" }, `${list.length} contracts`)),
    h("div", { class: "grid" }, ...list.map(contractCard))
  );
}

function contractCard(contract: Contract) {
  return h(
    contract.hasLayout ? "a" : "div",
    {
      class: `card${contract.hasLayout ? "" : " disabled"}`,
      href: contract.hasLayout ? `#/${encodeURIComponent(contract.name)}` : undefined
    },
    h("span", { class: "name" }, contract.name),
    h("code", { title: contract.packageHash }, short(contract.packageHash)),
    contract.hasLayout ? null : h("span", { class: "badge" }, "no layout")
  );
}

async function readerView(name: string) {
  show(h("p", { class: "muted" }, "Loading layout…"));
  let storage: ContractStorage;
  try {
    const contract = (await contracts).find((c) => c.name === name);
    if (!contract) throw new Error(`Unknown contract ${name}`);
    storage = await ContractStorage.open(contract);
  } catch (e) {
    return show(backLink(), errorBox(e));
  }

  const hash = h("code", {}, "resolving…");
  storage.contractHash.then(
    (value) => hash.replaceChildren(value),
    (e) => hash.replaceChildren(h("span", { class: "error-text" }, message(e)))
  );

  const panel = h("section", { class: "panel" }, h("p", { class: "muted" }, "Select a field."));
  const items = storage.fields().map((field) =>
    h(
      "button",
      {
        class: "field",
        onclick: (event: Event) => {
          for (const item of items) item.classList.remove("selected");
          (event.currentTarget as HTMLElement).classList.add("selected");
          panel.replaceChildren(...fieldPanel(storage, field));
        }
      },
      h("span", { class: "path" }, field.path),
      h("span", { class: "type" }, typeName(field.type)),
      field.keys.length ? h("span", { class: "badge" }, plural(field.keys.length, "key")) : null
    )
  );

  show(
    backLink(),
    h(
      "header",
      {},
      h("h1", {}, storage.contract.name),
      h(
        "dl",
        {},
        h("dt", {}, "Package"),
        h("dd", {}, h("code", {}, storage.contract.packageHash)),
        h("dt", {}, "Contract"),
        h("dd", {}, hash)
      )
    ),
    h("div", { class: "reader" }, h("nav", { class: "fields" }, ...items), panel)
  );
}

function fieldPanel(storage: ContractStorage, field: Field): Node[] {
  const inputs = field.keys.map((ty) => h("input", { placeholder: keyHint(ty), spellcheck: "false" }));
  const raw = h("input", { type: "checkbox" });
  const result = h("div", { class: "result" });

  const submit = async (event: Event) => {
    event.preventDefault();
    result.replaceChildren(h("p", { class: "muted" }, "Reading…"));
    try {
      const keys = inputs.map((input) => input.value.trim());
      const read = await storage.read(field.path, keys, raw.checked);
      result.replaceChildren(...readResult(read, raw.checked));
    } catch (e) {
      result.replaceChildren(errorBox(e));
    }
  };

  return [
    h("h2", {}, h("code", {}, field.path)),
    h("p", { class: "muted" }, `${typeName(field.type)} · ${storageLabel(field)}`),
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

function readResult({ location, value }: ReadResult, raw: boolean): Node[] {
  const where =
    location.kind === "dictionary"
      ? [
          h("dt", {}, "Dictionary"),
          h("dd", {}, h("code", {}, location.dictionaryName)),
          h("dt", {}, "Item key"),
          h("dd", {}, h("code", {}, location.dictionaryItemKey))
        ]
      : [h("dt", {}, "Named key"), h("dd", {}, h("code", {}, location.name))];
  return [
    h("dl", {}, ...where),
    value === null
      ? h("p", { class: "value unset" }, "Not set")
      : h("pre", { class: "value" }, raw ? `0x${value}` : value)
  ];
}

function backLink() {
  return h("a", { href: "#/", class: "back" }, "← Contracts");
}

function errorBox(e: unknown) {
  return h("div", { class: "error" }, message(e));
}

// ---------------------------------------------------------------------------------------------
// DOM helpers

type Child = Node | string | null | undefined | false;

// Creates an element; children are appended as nodes or text, never parsed as HTML.
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

function show(...children: Child[]) {
  app.replaceChildren(...(children.filter(Boolean) as (Node | string)[]));
}

const short = (hash: string) => hash.replace(/^(\w+-)?(.{8}).*(.{6})$/, "$1$2…$3");
const plural = (count: number, noun: string) => `${count} ${noun}${count > 1 ? "s" : ""}`;
const message = (e: unknown) => (e instanceof Error ? e.message : String(e));

// ---------------------------------------------------------------------------------------------
// Start

function route() {
  const name = decodeURIComponent(location.hash.replace(/^#\/?/, ""));
  if (name) readerView(name);
  else contractsView();
}

await loadReader();
contracts = fetchContracts();
window.addEventListener("hashchange", route);
route();
