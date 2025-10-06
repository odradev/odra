import init, {
    Address,
    AccountInfo,
    WCSPRClient,
    OdraWasmClient,
    U256,
    U512,
    TransactionStatus,
    TransactionResult,
    setGas,
    DEFAULT_PAYMENT_AMOUNT,
    WCSPRErrors,
    CsprClickCallbacks,
    getCurrentAccount
} from "odra-wasm-client";

// ---------- Types ----------
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
const addressDropdownBtn = document.getElementById("address-dropdown-btn") as HTMLButtonElement;
const addressDropdownMenu = document.getElementById("address-dropdown-menu") as HTMLDivElement;
const switchAccountBtn = document.getElementById("switch-account-btn") as HTMLButtonElement;
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
    await client.signIn();
  } catch (error) {
    showError("Failed to connect wallet.");
  }
}

async function onConnect(accountInfo: AccountInfo) {
  connected = true;
  address = accountInfo.publicKey;
  balances = {
    nativeCSPR: accountInfo.balance,
    wCSPR: U256.fromNumber(0)
  };
  addressSpan.textContent = `${address.slice(0, 5)}...${address.slice(-5)}`;
  connectBtn.classList.add("hidden");
  disconnectSection.classList.remove("hidden");
  await refreshBalances(accountInfo);
}

async function disconnect() {
  await client.signOut();
}

async function refreshBalances(accountInfo: AccountInfo) {
  if (!connected) return;
  nativeBalSpan.classList.add("hidden");
  wrappedBalSpan.classList.add("hidden");
  nativeBalLoader.classList.remove("hidden");
  wrappedBalLoader.classList.remove("hidden");
  try {
    const caller = accountInfo.address;
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
  if (balances) {
    if (direction === "NATIVE_TO_WRAPPED" && amount.gt(balances.nativeCSPR)) return null;
    if (direction === "WRAPPED_TO_NATIVE" && amount.gt(balances.wCSPR.toU512())) return null;
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

function clearUserData() {
  connected = false;
  address = null;
  balances = null;
  addressSpan.textContent = "";
  nativeBalSpan.textContent = "—";
  wrappedBalSpan.textContent = "—";
  txSection.classList.add("hidden");
  addressDropdownMenu.classList.add("hidden");
  connectBtn.classList.remove("hidden");
  disconnectSection.classList.add("hidden");
}

function onTransactionStatusUpdate(status: TransactionStatus, data: TransactionResult) {
  if (status === TransactionStatus.SENT) {
    txSection.classList.remove("hidden");
    txStatusDiv.textContent = "Transaction is being processed...";
    const url = `${EXPLORER_BASE.replace(/\/+$/, "")}/transaction/${data.transactionHash}`;
    txLinkAnchor.href = url;
  } else if (status === TransactionStatus.PROCESSED) {
    if (data.error) {
        if (data.errorCode) {
        if (data.errorCode === WCSPRErrors.CannotTargetSelfUser) {
          showError("Transaction failed: Cannot target yourself.");
        } else if (data.errorCode === WCSPRErrors.InsufficientAllowance) {
          showError("Transaction failed: Insufficient allowance approved.");
        } else if (data.errorCode === WCSPRErrors.InsufficientBalance) {
          showError("Transaction failed: Insuffcient balance.");
        } else {
          showError(`Transaction failed with error code: ${data.errorCode}`);
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
}

// ---------- Event listeners ----------
connectBtn.addEventListener("click", connect);
disconnectBtn.addEventListener("click", disconnect);
refreshBtn.addEventListener("click", async () => {
  const account = await getCurrentAccount();
  await refreshBalances(account);
});
swapBtn.addEventListener("click", onSwap);
dirNativeBtn.addEventListener("click", () => setDirection("NATIVE_TO_WRAPPED"));
dirWrappedBtn.addEventListener("click", () => setDirection("WRAPPED_TO_NATIVE"));

// Dropdown functionality
addressDropdownBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  addressDropdownMenu.classList.toggle("hidden");
});

switchAccountBtn.addEventListener("click", async () => {
  addressDropdownMenu.classList.add("hidden");
  try {
    await client.switchAccount();
  } catch (error) {
    console.error("Failed to switch account:", error);
    showError("Failed to switch account.");
  }
});

// Close dropdown when clicking outside
document.addEventListener("click", () => {
  addressDropdownMenu.classList.add("hidden");
});

// Initialize default state
setDirection("NATIVE_TO_WRAPPED");

async function run() {
    // 1. Initialize WASM
    await init();

    // 2. Initialize the clients
    const address = new Address("hash-8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675");
    client = new OdraWasmClient("https://testnet-rpc.odra.dev", "https://testnet-speculative-rpc.odra.dev", "casper-test");
    wcspr = new WCSPRClient(client, address);

    // 3. Set your custom callback
    CsprClickCallbacks.onSignedIn(async (accountInfo: AccountInfo) => {
        console.log('Signed in handler:', accountInfo);
        await onConnect(accountInfo);
    });
    CsprClickCallbacks.onSwitchedAccount(async (accountInfo: AccountInfo) => {
        console.log('Switched account handler:', accountInfo);
        await onConnect(accountInfo);
    });
    CsprClickCallbacks.onUnsolicitedAccountChange(async (accountInfo: AccountInfo) => {
        console.log('Unsolicited account change handler:', accountInfo);
    });
    CsprClickCallbacks.onSignedOut(() => {
        console.log('Signed out handler');
        clearUserData();
    });
    CsprClickCallbacks.onTransactionStatusUpdate((status: TransactionStatus, data: TransactionResult) => {
        console.log('Transaction status update handler:', status, data);
        onTransactionStatusUpdate(status, data);
    });
}

// 4. Start the initialization process
run().catch(err => console.error("Failed to initialize:", err));