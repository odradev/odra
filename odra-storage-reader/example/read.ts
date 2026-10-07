// Reads a value from the storage of a deployed Odra contract.
//
//   npm run read -- <layout.json> <package-hash> <path> [key...]
//   npm run read -- ../tests/fixtures/erc20_layout.json hash-f628... balances account-hash-9918...
//
// The layout file is the output of `odra-cli --json storage <Contract>`. The node is
// NODE_URL (default: NCTL). In a browser the reader calls are the same, only the wasm is
// loaded with `await init()` and the node is usually reached through a proxy (CORS).
import { readFileSync } from "node:fs";
import {
  HttpHandler,
  ParamDictionaryIdentifier,
  ParamDictionaryIdentifierContractNamedKey,
  RpcClient,
  RpcError
} from "casper-js-sdk";
import init, { StorageReader } from "odra-storage-reader";

const NODE_URL = process.env.NODE_URL ?? "http://localhost:11101/rpc";
// The node's answer for a key that was never written.
const QUERY_FAILED = -32003;

const [layoutPath, packageHash, path, ...keys] = process.argv.slice(2);
if (!layoutPath || !packageHash || !path) {
  console.error("usage: npm run read -- <layout.json> <package-hash> <path> [key...]");
  process.exit(1);
}

await init({
  module_or_path: readFileSync(new URL("../pkg-web/odra_storage_reader_bg.wasm", import.meta.url))
});
const reader = new StorageReader(readFileSync(layoutPath, "utf8"));
const rpc = new RpcClient(new HttpHandler(NODE_URL));

// 1. The current contract of the package. `rawJSON` is the node's JSON, which the reader takes.
const pkg = await rpc.queryLatestGlobalState(packageHash, []);
const contractHash = reader.currentContractHash(pkg.rawJSON.stored_value);

// 2. Where the value is stored.
const location = reader.locate(path, keys);

// 3. Fetch and decode it.
try {
  const result =
    location.kind === "dictionary"
      ? await rpc.getDictionaryItemByIdentifier(
          null,
          new ParamDictionaryIdentifier(
            undefined,
            new ParamDictionaryIdentifierContractNamedKey(
              contractHash,
              location.dictionaryName,
              location.dictionaryItemKey
            )
          )
        )
      : await rpc.queryLatestGlobalState(contractHash, [location.name]);
  const clValue = result.rawJSON.stored_value.CLValue;
  console.log(`${reader.contract}.${path} = ${reader.decode(clValue, location)}`);
} catch (e) {
  if (e instanceof RpcError && e.code === QUERY_FAILED) {
    console.log(`${reader.contract}.${path} = <not set>`);
  } else {
    throw e;
  }
}
