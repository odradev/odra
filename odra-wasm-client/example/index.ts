import { 
    GetStateRootHashResult,
    GetBalanceResult,
    OdraWasmClient,
    Address,
    Verbosity,
} from "odra-wasm-client";

let address = new Address("account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364");

const client = new OdraWasmClient("http://95.165.150.165:7777", Verbosity.High);

const hash: GetStateRootHashResult = await client.get_state_root_hash();
console.log("State root hash:", hash.state_root_hash_as_string);

const balance: GetBalanceResult = await client.get_balance(address);
console.log("Balance:", balance.balance_value);