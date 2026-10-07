# odra-storage-reader

A wasm library that reads the storage of a deployed Odra contract from a frontend, without
calling the contract.

It does not talk to the node. It tells you where a value is stored and decodes what you fetched
with your own RPC client (e.g. `casper-js-sdk`). It works for any contract that has a layout file.

## Layout file

Export it with the contract's `odra-cli`:

```sh
cargo run --bin odra_cli -- --json storage Erc20 > erc20_layout.json
```

## Usage

```ts
import init, { StorageReader } from "odra-storage-reader";

await init();
const reader = new StorageReader(layoutJson);

// The current contract of the package.
const pkg = await rpc.queryLatestGlobalState(packageHash, []);
const contractHash = reader.currentContractHash(pkg.rawJSON.stored_value);

// Where the value is stored: a dictionary item or a named key.
const location = reader.locate("balances", ["account-hash-9918…"]);
// { kind: "dictionary", dictionaryName: "state", dictionaryItemKey: "caebe9…", type: "U256" }

const item = await rpc.getDictionaryItemByIdentifier(null,
  new ParamDictionaryIdentifier(undefined,
    new ParamDictionaryIdentifierContractNamedKey(
      contractHash, location.dictionaryName, location.dictionaryItemKey)));
reader.decode(item.rawJSON.stored_value.CLValue, location);   // "10000"
```

- **Path:** dotted field names (`erc20.balances`). Use `len` for the length of a `List`.
- **Keys:** one per `Mapping`, `List` item or dictionary on the path, in path order. Write them
  in the `odra-cli` text format: `account-hash-…`, `some:5`, `owner:spender` for a tuple.
- **Named keys:** for a `{ kind: "named_key", name }` location, read it with
  `queryLatestGlobalState(contractHash, [name])`.
- **Values:** `decode` returns the value formatted like `odra-cli` does. `{ raw: true }` returns
  the stored bytes as hex instead.
- **Unwritten keys:** for a key that was never written, the node answers `-32003 Query failed`.
- **Errors:** errors are thrown as `JsError`s, with the same messages as `odra-cli`.

[`example/read.ts`](example/read.ts) shows the full flow against a node.

## Build

```sh
just prepare   # installs wasm-pack
just build     # pkg-web/
cd example && npm install
npm run read -- ../tests/fixtures/erc20_layout.json <package-hash> balances <account-hash>
```

## Limitations

- **Custom types:** only types that appear in entry points or events are in the layout file. A
  struct or enum that is only stored can't be decoded; use `{ raw: true }` for it.
- **Value format:** values come back as `odra-cli` formatted strings, not structured JSON.
- **Copied code:** the key parsing and value decoding are copied from `odra-cli/src/types`.
