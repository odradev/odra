import init, {
    Address,
    WCSPRClient,
    OdraWasmClient,
    U256,
    U512,
    TransactionStatus,
    setGas,
    DEFAULT_PAYMENT_AMOUNT,
    WCSPRErrors,
    CsprClickCallbacks
} from "odra-wasm-client";

// ---------- Types ----------
let wcspr: WCSPRClient;
let client: OdraWasmClient;

interface Balances {
  nativeCSPR: U512;
  wCSPR: U256;
}

interface AccountInfo {
  origin: string;
  account: {
    provider: string;
    providerSupports: string[];
    name: string | null;
    public_key: string;
    connected_at: number;
    token: string | null;
    last_used: number;
    cspr_name: string | null;
    fiat_cspr_rate: number;
    fiat_currency: string;
    balance: string;
    liquid_balance: string;
  };
}

declare global {
    interface Window {
        csprclick?: CsprClick;
    }
}

interface CsprClick {
  signIn(): Promise<void>;
  signOut(): void;
  isSignedIn(): boolean;
  getActivePublicKey(): Promise<string>;
  getAccounts(): Promise<Array<{publicKey: string, name: string | null}>>;
  signInWithAccount(account: {publicKey: string, name: string | null}): Promise<void>;
  signMessage(message: string, signingPublicKey: string): Promise<{signature: string, publicKey: string} | undefined>;
}

// ---------- Configuration ----------
const EXPLORER_BASE = "https://testnet.cspr.live/";
const TOKEN_DECIMALS = 9;
const DEPOSIT_GAS_AMOUNT = BigInt(7_000_000_000); // 7 CSPR
const WITHDRAW_GAS_AMOUNT = BigInt(3_000_000_000); // 3 CSPR

// ---------- State ----------
let connected = false;
let address: string | null = null;
let balances: Balances | null = null;
let direction: "NATIVE_TO_WRAPPED" | "WRAPPED_TO_NATIVE" = "NATIVE_TO_WRAPPED";

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
const txStatusDiv = document.getElementById("tx-status") as HTMLDivElement;
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
    await window.csprclick?.signIn();
  } catch (error) {
    showError("Failed to connect wallet.");
  }
}

async function onConnect(accountInfo: AccountInfo) {
  connected = true;
  address = accountInfo.account.public_key;
  const balance = BigInt(accountInfo.account.balance);
  balances = {
    nativeCSPR: U512.fromBigInt(balance),
    wCSPR: U256.fromNumber(0)
  };
  addressSpan.textContent = `${address.slice(0, 5)}...${address.slice(-5)}`;
  connectBtn.classList.add("hidden");
  disconnectBtn.classList.remove("hidden");
  disconnectSection.classList.remove("hidden");
  await refreshBalances(accountInfo);
}

async function disconnect() {
  await client.disconnect();
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

async function refreshBalances(accountInfo: AccountInfo) {
  if (!connected) return;
  nativeBalSpan.classList.add("hidden");
  wrappedBalSpan.classList.add("hidden");
  nativeBalLoader.classList.remove("hidden");
  wrappedBalLoader.classList.remove("hidden");
  try {
    const caller = Address.fromPublicKey(accountInfo.account.public_key);
    const wcsprBalance = await wcspr.balanceOf(caller);
    balances = {
      nativeCSPR: balances!.nativeCSPR,
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
  const amount = U512.fromHtmlInput(amountInput).mul(U512.fromNumber(1_000_000_000)); // Convert to smallest unit
  // if (balances) {
  //   if (direction === "NATIVE_TO_WRAPPED" && amount.gt(balances.nativeCSPR)) return null;
  //   if (direction === "WRAPPED_TO_NATIVE" && amount.gt(balances.wCSPR.toU512())) return null;
  // }
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
    if (direction === "NATIVE_TO_WRAPPED") {
      setGas(DEFAULT_PAYMENT_AMOUNT());
      await wcspr.deposit(amt);
    } else {
      setGas(WITHDRAW_GAS_AMOUNT);
      await wcspr.withdraw(U256.fromU512(amt));
    }
    amountInput.value = "";
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
// refreshBtn.addEventListener("click", refreshBalances);
swapBtn.addEventListener("click", onSwap);
dirNativeBtn.addEventListener("click", () => setDirection("NATIVE_TO_WRAPPED"));
dirWrappedBtn.addEventListener("click", () => setDirection("WRAPPED_TO_NATIVE"));

// Initialize default state
setDirection("NATIVE_TO_WRAPPED");

async function run() {
    // 1. Initialize WASM
    await init();

    // 2. Initialize the clients
    const address = new Address("hash-8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675");
    client = new OdraWasmClient("https://testnet-rpc.odra.dev", "https://testnet-speculative-rpc.odra.dev", "casper-test");
    wcspr = new WCSPRClient(client, address);

    // Set your custom callback
    CsprClickCallbacks.onSignedIn(async (accountInfo: AccountInfo) => {
        console.log('Rust  signe in handler:', accountInfo);
        await onConnect(accountInfo);
    });
    CsprClickCallbacks.onSwitchedAccount(async (event: any) => {
    });
    CsprClickCallbacks.onSignedOut(async (event: any) => {
        disconnect();
    });
    CsprClickCallbacks.onTransactionStatusUpdate((status: TransactionStatus, data: any) => {
        console.log('Rust transaction status update handler:', status, data);
        if (status === TransactionStatus.SENT) {
          txSection.classList.remove("hidden");
          txStatusDiv.textContent = "Transaction is being processed...";
          const url = `${EXPLORER_BASE.replace(/\/+$/, "")}/transaction/${data.transactionHash}`;
          txLinkAnchor.href = url;
        } else if (status === TransactionStatus.PROCESSED) {
          if (data.error) {
             if (data.odraErrorCode) {
              if (data.odraErrorCode === WCSPRErrors.CannotTargetSelfUser) {
                showError("Transaction failed: Cannot target yourself.");
              } else if (data.odraErrorCode === WCSPRErrors.InsufficientAllowance) {
                showError("Transaction failed: Insufficient allowance approved.");
              } else if (data.odraErrorCode === WCSPRErrors.InsufficientBalance) {
                showError("Transaction failed: Insuffcient balance.");
              } else {
                showError(`Transaction failed with error code: ${data.odraErrorCode}`);
              }
            } else {
              showError("Transaction failed with unknown error.");
            }
          } else {
            txStatusDiv.textContent = "Transaction succeeded.";
          }
        } else if (status === TransactionStatus.ERROR) {
          txSection.classList.add("hidden");
          showError("Transaction failed with unknown error.");
        }
        // Your custom logic here
    });
}

// 3. Start the initialization process
run().catch(err => console.error("Failed to initialize:", err));