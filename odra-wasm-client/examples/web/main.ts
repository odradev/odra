import init, {
    Address,
    OdraWasmClient,
    Verbosity,
    Cep18Client,
    U256
} from "odra-wasm-client";

async function runAction() {
    // Implement your action here
    const address = new Address("hash-b69714753812df8edcbd38aec02b9f86ab44a30891050ed331ac2f40ef7b2580");
    const client = new OdraWasmClient("http://95.165.150.165:7777", Verbosity.High);

    const cep18 = new Cep18Client(client, address);
    
    const spender = new Address("account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364");
    const value = new U256("1000000000");
    let hash = await cep18.approve(spender, value);
    console.log("Approval transaction hash:", hash);
}

async function run() {
    await init();
    console.log('WASM module initialized');
    document.getElementById('run-button')?.addEventListener('click', runAction);
}

run();
