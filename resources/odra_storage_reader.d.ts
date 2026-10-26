/* tslint:disable */
/* eslint-disable */
/**
 * Reads the storage of a deployed Odra contract, see the crate docs.
 */
export class StorageReader {
  free(): void;
  /**
   * The hex-encoded bytes of the value read from `location`, without decoding them, e.g. for a
   * type that is not in the schema file. Takes the same arguments as `decode`.
   */
  decodeRaw(clValue: any, location: any): string;
  /**
   * Every value that can be read: `{ path, keys, type, storage }`, with the path and the key
   * types as `locate` takes them.
   */
  readableFields(): any;
  /**
   * The hash (`hash-...`) of the current version of a contract package, from the
   * `rawJSON.stored_value` of `queryLatestGlobalState(packageHash, [])`.
   */
  currentContractHash(stored_value: any): string;
  /**
   * Creates a reader from the contract's schema file (the `*_schema.json` `odra-build`
   * writes), given as a JSON string or an object.
   */
  constructor(schema: any);
  /**
   * Decodes the value read from `location` (the result of `locate`). `clValue` is the
   * `rawJSON.stored_value.CLValue` of the RPC result.
   */
  decode(clValue: any, location: any): string;
  /**
   * The storage layout tree of the contract.
   */
  fields(): any;
  /**
   * Resolves a dotted field path to the location of the value. `keys` are the keys of the
   * mappings, list items and dictionaries on the path, in path order.
   */
  locate(path: string, keys?: string[] | null): any;
  /**
   * The name of the contract.
   */
  readonly contract: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_storagereader_free: (a: number, b: number) => void;
  readonly storagereader_contract: (a: number) => [number, number];
  readonly storagereader_currentContractHash: (a: number, b: any) => [number, number, number, number];
  readonly storagereader_decode: (a: number, b: any, c: any) => [number, number, number, number];
  readonly storagereader_decodeRaw: (a: number, b: any, c: any) => [number, number, number, number];
  readonly storagereader_fields: (a: number) => [number, number, number];
  readonly storagereader_locate: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly storagereader_new: (a: any) => [number, number, number];
  readonly storagereader_readableFields: (a: number) => [number, number, number];
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
