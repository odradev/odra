import { 
    GetStateRootHashResult,
    GetBalanceResult,
    OdraWasmClient,
    Verbosity,
} from "odra-wasm-client";

const client = new OdraWasmClient("http://95.165.150.165:7777", Verbosity.High);

const hash: GetStateRootHashResult = await client.get_state_root_hash();
console.log("State root hash:", hash.state_root_hash_as_string);

const balance: GetBalanceResult = await client.get_balance(
    "uref-d72643d4a776e6041c3d9229ff9828c3559c1b2c85ef4dec1a4a37a27241f4ef-007"
);
console.log("Balance:", balance.balance_value);