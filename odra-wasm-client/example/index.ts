import { 
    GetStateRootHashResult,
    GetBalanceResult,
    OdraWasmClient,
    Address,
    Verbosity,
    Bytes,
    get_balance
} from "odra-wasm-client";

// let address = new Address("account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364");

// const client = new OdraWasmClient("http://95.165.150.165:7777", Verbosity.High);

// const hash: GetStateRootHashResult = await client.get_state_root_hash();
// console.log("State root hash:", hash.state_root_hash_as_string);

// const balance: GetBalanceResult = await client.get_balance(address);
// console.log("Balance:", balance.balance_value);

// const contractAddress = new Address("hash-7c5e3793decb7a3c7f1fb8b08e5625b6e545229fc9e82936d864918180d50374");
// const decimals: Bytes | undefined = await client.getNamedValue(contractAddress, "__events_length");

// console.log("Decimals:", decimals?.toString());

let address = new Address("account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364");
console.log("Balance:", get_balance(address));
