var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import init, { Address, Cep18Client, OdraWasmClient, U256 } from "wasm-client";
let client;
let cep18;
function setText(id, text) {
    const el = document.getElementById(id);
    if (el)
        el.innerHTML = text;
}
function approve() {
    return __awaiter(this, void 0, void 0, function* () {
        const spender = new Address(document.getElementById('approve-spender').value);
        const amount = new U256(document.getElementById('approve-amount').value);
        const hash = yield cep18.approve(spender, amount);
        console.log(hash);
    });
}
function transfer() {
    return __awaiter(this, void 0, void 0, function* () {
        const recipient = new Address(document.getElementById('transfer-recipient').value);
        const amount = new U256(document.getElementById('transfer-amount').value);
        const hash = yield cep18.transfer(recipient, amount);
        console.log(hash);
    });
}
function transferFrom() {
    return __awaiter(this, void 0, void 0, function* () {
        const owner = new Address(document.getElementById('transfer-from-owner').value);
        const recipient = new Address(document.getElementById('transfer-from-recipient').value);
        const amount = new U256(document.getElementById('transfer-from-amount').value);
        const hash = yield cep18.transferFrom(owner, recipient, amount);
        console.log(hash);
    });
}
function getName() {
    return __awaiter(this, void 0, void 0, function* () {
        const name = yield cep18.name();
        setText('token-name', name);
    });
}
function getSymbol() {
    return __awaiter(this, void 0, void 0, function* () {
        const symbol = yield cep18.symbol();
        setText('token-symbol', symbol);
    });
}
function getDecimals() {
    return __awaiter(this, void 0, void 0, function* () {
        const decimals = yield cep18.decimals();
        setText('token-decimals', decimals.toString());
    });
}
function getTotalSupply() {
    return __awaiter(this, void 0, void 0, function* () {
        const totalSupply = yield cep18.totalSupply();
        setText('total-supply', totalSupply.toString());
    });
}
function getBalanceOf() {
    return __awaiter(this, void 0, void 0, function* () {
        const address = new Address(document.getElementById('balance-of-address').value);
        const balance = yield cep18.balanceOf(address);
        setText('balance-of-result', balance.toString());
    });
}
function getAllowance() {
    return __awaiter(this, void 0, void 0, function* () {
        const owner = new Address(document.getElementById('allowance-owner').value);
        const spender = new Address(document.getElementById('allowance-spender').value);
        const allowance = yield cep18.allowance(owner, spender);
        setText('allowance-result', allowance.toString());
    });
}
/**
 * Waits for CasperWalletProvider to be available on window
 */
function waitForCasperWalletProvider(timeout = 10000) {
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
function run() {
    return __awaiter(this, void 0, void 0, function* () {
        var _a, _b, _c, _d, _e, _f, _g, _h, _j;
        // 1. Initialize WASM
        yield init();
        console.log('WASM module initialized');
        // 2. Wait for wallet provider and connect
        try {
            const provider = yield waitForCasperWalletProvider();
            console.log('CasperWalletProvider found:', provider);
        }
        catch (error) {
            console.warn('No wallet extension detected:', error);
        }
        // 3. Initialize the clients
        client = new OdraWasmClient("https://testnet-rpc.odra.dev", "https://testnet-speculative-rpc.odra.dev", "casper-test");
        // const address = new Address("hash-2879d6e927289197aab0101cc033f532fe22e4ab4686e44b5743cb1333031acc");
        const address = new Address("hash-b69714753812df8edcbd38aec02b9f86ab44a30891050ed331ac2f40ef7b2580");
        cep18 = new Cep18Client(client, address);
        // 4. Set up event listeners after clients are ready
        (_a = document.getElementById('get-name-button')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', getName);
        (_b = document.getElementById('get-symbol-button')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', getSymbol);
        (_c = document.getElementById('get-decimals-button')) === null || _c === void 0 ? void 0 : _c.addEventListener('click', getDecimals);
        (_d = document.getElementById('get-total-supply-button')) === null || _d === void 0 ? void 0 : _d.addEventListener('click', getTotalSupply);
        (_e = document.getElementById('balance-of-button')) === null || _e === void 0 ? void 0 : _e.addEventListener('click', getBalanceOf);
        (_f = document.getElementById('approve-button')) === null || _f === void 0 ? void 0 : _f.addEventListener('click', approve);
        (_g = document.getElementById('transfer-button')) === null || _g === void 0 ? void 0 : _g.addEventListener('click', transfer);
        (_h = document.getElementById('transfer-from-button')) === null || _h === void 0 ? void 0 : _h.addEventListener('click', transferFrom);
        (_j = document.getElementById('allowance-button')) === null || _j === void 0 ? void 0 : _j.addEventListener('click', getAllowance);
    });
}
// 5. Start the initialization process
run().catch(err => console.error("Failed to initialize:", err));
//# sourceMappingURL=main.js.map