import init, {
    Address,
    Cep18Client,
    OdraWasmClient,
    U256
} from "wasm-client";

let client: OdraWasmClient;
let cep18: Cep18Client;

function setText(id: string, text: string) {
    const el = document.getElementById(id);
    if (el) el.innerHTML = text;
}

async function approve() {
    const spender = new Address((document.getElementById('approve-spender') as HTMLInputElement).value);
    const amount = new U256((document.getElementById('approve-amount') as HTMLInputElement).value);
    const hash = await cep18.approve(spender, amount);
    console.log(hash);
}

async function transfer() {
    const recipient = new Address((document.getElementById('transfer-recipient') as HTMLInputElement).value);
    const amount = new U256((document.getElementById('transfer-amount') as HTMLInputElement).value);
    const hash = await cep18.transfer(recipient, amount);
    console.log(hash);
}

async function transferFrom() {
    const owner = new Address((document.getElementById('transfer-from-owner') as HTMLInputElement).value);
    const recipient = new Address((document.getElementById('transfer-from-recipient') as HTMLInputElement).value);
    const amount = new U256((document.getElementById('transfer-from-amount') as HTMLInputElement).value);
    const hash = await cep18.transferFrom(owner, recipient, amount);
    console.log(hash);
}

async function getName() {
    const name = await cep18.name();
    setText('token-name', name);
}

async function getSymbol() {
    const symbol = await cep18.symbol();
    setText('token-symbol', symbol);
}

async function getDecimals() {
    const decimals = await cep18.decimals();
    setText('token-decimals', decimals.toString());
}

async function getTotalSupply() {
    const totalSupply = await cep18.totalSupply();
    setText('total-supply', totalSupply.toString());
}

async function getBalanceOf() {
    const address = new Address((document.getElementById('balance-of-address') as HTMLInputElement).value);
    const balance = await cep18.balanceOf(address);
    setText('balance-of-result', balance.toString());
}

async function getAllowance() {
    const owner = new Address((document.getElementById('allowance-owner') as HTMLInputElement).value);
    const spender = new Address((document.getElementById('allowance-spender') as HTMLInputElement).value);
    const allowance = await cep18.allowance(owner, spender);
    setText('allowance-result', allowance.toString());
}

/**
 * Waits for CasperWalletProvider to be available on window
 */
function waitForCasperWalletProvider(timeout = 10000): Promise<any> {
    return new Promise((resolve, reject) => {
        // Check if already available
        if (window.CasperWalletProvider) {
            return resolve(window.CasperWalletProvider);
        }

        const startTime = Date.now();

        const checkWallet = () => {
            if (window.CasperWalletProvider) {
                resolve(window.CasperWalletProvider);
                return;
            }

            if (Date.now() - startTime > timeout) {
                reject(new Error('CasperWalletProvider not available. Is the extension installed?'));
                return;
            }

            setTimeout(checkWallet, 100);
        };

        checkWallet();
    });
}

// Make TypeScript aware of the global object
declare global {
    interface Window {
        CasperWalletProvider?: any;
    }
}

async function run() {
    // 1. Initialize WASM
    await init();
    console.log('WASM module initialized');

    // 2. Wait for wallet provider and connect
    try {
        const provider = await waitForCasperWalletProvider();
        console.log('CasperWalletProvider found:', provider);
    } catch (error) {
        console.warn('No wallet extension detected:', error);
    }

    // 3. Initialize the clients
    client = new OdraWasmClient("https://testnet-rpc.odra.dev", "https://testnet-speculative-rpc.odra.dev", "casper-test");
    // const address = new Address("hash-2879d6e927289197aab0101cc033f532fe22e4ab4686e44b5743cb1333031acc");
    const address = new Address("hash-b69714753812df8edcbd38aec02b9f86ab44a30891050ed331ac2f40ef7b2580");
    cep18 = new Cep18Client(client, address);

    // 4. Set up event listeners after clients are ready
    document.getElementById('get-name-button')?.addEventListener('click', getName);
    document.getElementById('get-symbol-button')?.addEventListener('click', getSymbol);
    document.getElementById('get-decimals-button')?.addEventListener('click', getDecimals);
    document.getElementById('get-total-supply-button')?.addEventListener('click', getTotalSupply);
    document.getElementById('balance-of-button')?.addEventListener('click', getBalanceOf);
    document.getElementById('approve-button')?.addEventListener('click', approve);
    document.getElementById('transfer-button')?.addEventListener('click', transfer);
    document.getElementById('transfer-from-button')?.addEventListener('click', transferFrom);
    document.getElementById('allowance-button')?.addEventListener('click', getAllowance);
}

// 5. Start the initialization process
run().catch(err => console.error("Failed to initialize:", err));