import * as wasm from "client";

async function test() {
    console.log(await wasm.test_client());
}

test();
console.log(wasm.schemas());