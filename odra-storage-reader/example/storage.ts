// How the explorer reads contract storage with odra-storage-reader and casper-js-sdk.
//
// This is the only file that uses the reader or talks to the node; the rest of the app is
// presentation. A read takes five steps, numbered below.
import type * as Casper from "casper-js-sdk";
import init, { StorageReader } from "odra-storage-reader";

import type { Field, Ty } from "./layout.js";

// The browser build of casper-js-sdk (loaded in index.html) defines its classes as globals.
const sdk = globalThis as unknown as typeof Casper;
// The node, through the proxy of server.ts (the node sends no CORS headers).
const rpc = new sdk.RpcClient(new sdk.HttpHandler(`${location.origin}/rpc`));
// The node's answer for a value that was never written.
const QUERY_FAILED = -32003;
// casper-js-sdk throws an `HttpError` with the RPC error as `sourceErr`.
const isNotSet = (e: any) => (e?.sourceErr?.code ?? e?.code) === QUERY_FAILED;

/** A contract deployed with odra-cli. */
export interface Contract {
  name: string;
  packageHash: string;
  /** Whether `just export-layouts` wrote its layout file. */
  hasLayout: boolean;
}

/** Where a value is stored, as `StorageReader.locate` returns it. */
export type Location =
  | { kind: "dictionary"; dictionaryName: string; dictionaryItemKey: string; type: Ty }
  | { kind: "named_key"; name: string; type: Ty };

export interface ReadResult {
  location: Location;
  /** The decoded value (hex with `raw`), `null` if it was never written. */
  value: string | null;
}

/** Loads the wasm module, once, before anything else. */
export async function loadReader(): Promise<void> {
  await init();
}

/** The contracts deployed with odra-cli, listed by server.ts. */
export async function fetchContracts(): Promise<Contract[]> {
  const response = await fetch("/api/contracts");
  const body = await response.json();
  if (!response.ok) throw new Error(body.error);
  return body;
}

const opened = new Map<string, Promise<ContractStorage>>();

/** The storage of one contract. */
export class ContractStorage {
  private constructor(
    readonly contract: Contract,
    private readonly reader: StorageReader,
    /** The hash of the current contract version, the one to query. */
    readonly contractHash: Promise<string>
  ) {}

  /** Opens the storage of `contract`, once per contract. */
  static open(contract: Contract): Promise<ContractStorage> {
    let storage = opened.get(contract.name);
    if (!storage) {
      storage = ContractStorage.load(contract);
      storage.catch(() => opened.delete(contract.name));
      opened.set(contract.name, storage);
    }
    return storage;
  }

  private static async load(contract: Contract): Promise<ContractStorage> {
    // 1. The reader is created from the layout file, `odra-cli --json storage <Contract>`.
    const response = await fetch(`layouts/${contract.name}.json`);
    if (!response.ok) throw new Error(`No layout for ${contract.name}, run \`just export-layouts\``);
    const reader = new StorageReader(await response.text());

    // 2. The package resolves to its current contract, the one that holds the storage.
    const contractHash = rpc
      .queryLatestGlobalState(contract.packageHash, [])
      .then((pkg) => reader.currentContractHash(pkg.rawJSON.stored_value));

    return new ContractStorage(contract, reader, contractHash);
  }

  /** Every value that can be read, with its path and the types of the keys it needs. */
  fields(): Field[] {
    return this.reader.readableFields();
  }

  /**
   * Reads the value at `path`. `keys` are the keys of the mappings, list items and dictionaries
   * on the path, in path order, in the odra-cli text format. With `raw`, returns the stored bytes
   * as hex instead of decoding them.
   */
  async read(path: string, keys: string[], raw = false): Promise<ReadResult> {
    // 3. The path and keys resolve to where the value is stored: a dictionary item or a named key.
    const location: Location = this.reader.locate(path, keys);

    // 4. The node returns the stored value; `rawJSON` is the node's JSON, which the reader takes.
    let clValue;
    try {
      const result = await this.fetch(location);
      clValue = result.rawJSON.stored_value.CLValue;
    } catch (e) {
      if (isNotSet(e)) return { location, value: null };
      throw e;
    }

    // 5. The reader decodes it with the layout's types.
    const value = raw
      ? this.reader.decodeRaw(clValue, location)
      : this.reader.decode(clValue, location);
    return { location, value };
  }

  private async fetch(location: Location) {
    const contractHash = await this.contractHash;
    if (location.kind === "named_key") {
      return rpc.queryLatestGlobalState(contractHash, [location.name]);
    }
    return rpc.getDictionaryItemByIdentifier(
      null,
      new sdk.ParamDictionaryIdentifier(
        undefined,
        new sdk.ParamDictionaryIdentifierContractNamedKey(
          contractHash,
          location.dictionaryName,
          location.dictionaryItemKey
        )
      )
    );
  }
}
