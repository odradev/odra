import * as wasm from "client";

// convert now to bigint
let now = BigInt(Date.now());
console.log("Deploying with timestamp:");
console.log(now);
wasm.deploy_contract(now).then((address) => {
    console.log("contract address:");
    console.log(address);
});
