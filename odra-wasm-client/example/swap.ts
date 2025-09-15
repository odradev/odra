import { on } from "events";
import init, {
    Address,
    WCSPRClient,
    OdraWasmClient,
    CasperWallet,
    U256,
    U512,
    TransactionHash
} from "odra-wasm-client";

// ---------- Types ----------
let wallet: CasperWallet;
let wcspr: WCSPRClient;
let client: OdraWasmClient;

interface Balances {
  nativeCSPR: U512;
  wCSPR: U256;
}

// ---------- Configuration ----------
const EXPLORER_BASE = "https://testnet.cspr.live/";
const TOKEN_DECIMALS = 9;
const DEPOSIT_GAS_AMOUNT = BigInt(7_000_000_000); // 7 CSPR
const WITHDRAW_GAS_AMOUNT = BigInt(2_500_000_000); // 2.5 CSPR

// ---------- State ----------
let connected = false;
let address: string | null = null;
let balances: Balances | null = null;
let direction: "NATIVE_TO_WRAPPED" | "WRAPPED_TO_NATIVE" = "NATIVE_TO_WRAPPED";
let txLink: string | null = null;

// ---------- DOM Elements ----------
const connectBtn = document.getElementById("connect-btn") as HTMLButtonElement;
const disconnectBtn = document.getElementById("disconnect-btn") as HTMLButtonElement;
const disconnectSection = document.getElementById('disconnect-section') as HTMLDivElement;
const addressSpan = document.getElementById("address") as HTMLSpanElement;
const nativeBalSpan = document.getElementById("native-bal") as HTMLSpanElement;
const wrappedBalSpan = document.getElementById("wrapped-bal") as HTMLSpanElement;
const amountInput = document.getElementById("amount") as HTMLInputElement;
const swapBtn = document.getElementById("swap-btn") as HTMLButtonElement;
const refreshBtn = document.getElementById("refresh-btn") as HTMLButtonElement;
const txSection = document.getElementById("tx-section") as HTMLDivElement;
const txLinkAnchor = document.getElementById("tx-link") as HTMLAnchorElement;
const errorSection = document.getElementById("error-section") as HTMLDivElement;
const errorText = document.getElementById("error-text") as HTMLDivElement;
const dirNativeBtn = document.getElementById("dir-native") as HTMLButtonElement;
const dirWrappedBtn = document.getElementById("dir-wrapped") as HTMLButtonElement;
const nativeBalLoader = document.getElementById("native-bal-loader") as HTMLDivElement;
const wrappedBalLoader = document.getElementById("wrapped-bal-loader") as HTMLDivElement;

// ---------- Functions ----------
async function connect() {
  clearError();
  try {
    await wallet.connect();
    onConnect();
  } catch (error) {
      showError("Failed to connect wallet.");
  }
}

async function onConnect() {
  connected = true;
  address = await wallet.getActivePublicKey();
  addressSpan.textContent = `${address.slice(0, 5)}...${address.slice(-5)}`;
  connectBtn.classList.add("hidden");
  disconnectBtn.classList.remove("hidden");
  disconnectSection.classList.remove("hidden");
  await refreshBalances();
}

async function disconnect() {
  await wallet.disconnect();
  connected = false;
  address = null;
  balances = null;
  addressSpan.textContent = "";
  nativeBalSpan.textContent = "—";
  wrappedBalSpan.textContent = "—";
  txSection.classList.add("hidden");
  connectBtn.classList.remove("hidden");
  disconnectBtn.classList.add("hidden");
  disconnectSection.classList.add("hidden");
}

async function refreshBalances() {
  if (!connected) return;
  nativeBalSpan.classList.add("hidden");
  wrappedBalSpan.classList.add("hidden");
  nativeBalLoader.classList.remove("hidden");
  wrappedBalLoader.classList.remove("hidden");
  try {
    const caller: Address = await client.caller(wallet);
    const balance = await client.getBalance(caller);
    const wcsprBalance = await wcspr.balanceOf(caller);
    balances = {
      nativeCSPR: balance,
      wCSPR: wcsprBalance
    };
    nativeBalSpan.textContent = balances.nativeCSPR.formatter(TOKEN_DECIMALS).fmtWithPrecision(4);
    wrappedBalSpan.textContent = balances.wCSPR.formatter(TOKEN_DECIMALS).fmtWithPrecision(4);
  } catch (e: any) {
    console.error(e);
    showError("Failed to fetch balances");
  } finally {
    nativeBalSpan.classList.remove("hidden");
    wrappedBalSpan.classList.remove("hidden");
    nativeBalLoader.classList.add("hidden");
    wrappedBalLoader.classList.add("hidden");
  }
}

function validateAmount(): U512 | null {
  const amount = U512.fromHtmlInput(amountInput).mulBigInt(BigInt(1_000_000_000)); // Convert to smallest unit
  console.log("Validating amount:", amount.toString());
  console.log("Current balances:", balances?.nativeCSPR.toString(), balances?.wCSPR.toString());
  if (balances) {
    if (direction === "NATIVE_TO_WRAPPED" && amount > balances.nativeCSPR) return null;
    if (direction === "WRAPPED_TO_NATIVE" && amount > balances.wCSPR) return null;
  }
  return amount;
}

async function onSwap() {
  clearError();
  txSection.classList.add("hidden");
  const amt = validateAmount();
  if (amt === null) {
    showError("Invalid amount or insufficient balance.");
    return;
  }
  try {
    let result: TransactionHash;
    if (direction === "NATIVE_TO_WRAPPED") {
      // wcspr.set_gas(DEPOSIT_GAS_AMOUNT);
      result = await wcspr.deposit(amt);
    } else {
      // wcspr.set_gas(WITHDRAW_GAS_AMOUNT);
      result = await wcspr.withdraw(U256.fromU512(amt));
    }
    const url = `${EXPLORER_BASE.replace(/\/+$/, "")}/transaction/${result.toString()}`;
    txLink = url;
    txLinkAnchor.href = url;
    txSection.classList.remove("hidden");
    amountInput.value = "";
    await refreshBalances();
  } catch (e: any) {
    showError(e || "Transaction failed or was rejected");
  }
}

function setDirection(newDir: "NATIVE_TO_WRAPPED" | "WRAPPED_TO_NATIVE") {
  direction = newDir;
  if (direction === "NATIVE_TO_WRAPPED") {
    dirNativeBtn.classList.add("bg-indigo-100");
    dirWrappedBtn.classList.remove("bg-indigo-100");
  } else {
    dirWrappedBtn.classList.add("bg-indigo-100");
    dirNativeBtn.classList.remove("bg-indigo-100");
  }
}

function showError(msg: string) {
  errorText.textContent = msg;
  errorSection.classList.remove("hidden");
}

function clearError() {
  errorSection.classList.add("hidden");
  errorText.textContent = "";
}

// ---------- Event listeners ----------
connectBtn.addEventListener("click", connect);
disconnectBtn.addEventListener("click", disconnect);
refreshBtn.addEventListener("click", refreshBalances);
swapBtn.addEventListener("click", onSwap);
dirNativeBtn.addEventListener("click", () => setDirection("NATIVE_TO_WRAPPED"));
dirWrappedBtn.addEventListener("click", () => setDirection("WRAPPED_TO_NATIVE"));

// Initialize default state
setDirection("NATIVE_TO_WRAPPED");

declare global {
    interface Window {
        CasperWalletProvider?: any;
    }
}

/**
 * Waits for CasperWalletProvider to be available on window
 */
function waitForCasperWalletProvider(timeout = 10000): Promise<any> {
    return new Promise((resolve, reject) => {
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
    const address = new Address("hash-8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675");
    client = new OdraWasmClient("https://testnet-rpc.odra.dev", "https://testnet-speculative-rpc.odra.dev", "casper-test");
    wcspr = new WCSPRClient(client, address);
    wallet = new CasperWallet();

    if (await wallet.isConnected()) {
        await onConnect();
    }
}

// 4. Start the initialization process
run().catch(err => console.error("Failed to initialize:", err));