use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub(crate) type CasperWalletProvider;

    #[wasm_bindgen(js_name = CasperWalletProvider, catch)]
    pub(crate) fn casper_wallet_provider() -> Result<CasperWalletProvider, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn requestConnection(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn sign(
        this: &CasperWalletProvider,
        deploy: &str,
        signing_public_key_hex: &str
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn signMessage(
        this: &CasperWalletProvider,
        message: &str,
        signing_public_key_hex: &str
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn requestSwitchAccount(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn disconnectFromSite(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn isConnected(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn getActivePublicKey(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn getVersion(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error(s: &str);

    // pub(crate) type CsprClick;
    // pub(crate) type AccountType;

    // /// Call the connect() method using a provider name as the first parameter to request a connection using that wallet or login mechanism.
    // ///
    // /// Some providers may need an options argument to indicate the connection behavior requested.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick")]
    // pub(crate) fn connect(this: &CsprClick, provider: &str, options: &JsValue) -> Result<Promise, JsValue>;

    // /// Usually you will call signOut() method to close a user session.
    // /// Use disconnect() when you want to clear the connection between the wallet and your app.
    // /// Next time the user signs in with that wallet, he'll must gran connection permission again.
    // /// Send empty arguments or empty string to disconnect from currently active account.
    // /// Or call disconnect with a wallet provider key to force the disconnection of a specific wallet.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick")]
    // pub(crate) fn disconnect(this: &CsprClick, from_wallet: &str, options: &JsValue) -> Result<(), JsValue>;

    // /// Removes an account from the list of known accounts in CSPR.click.
    // /// It won’t be returned to the list of known accounts unless it’s connected again using the connect() method.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "forgetAccount")]
    // pub(crate) fn forget_account(this: &CsprClick, account_type: &JsValue) -> Result<Promise, JsValue>;

    // /// Gets the account for the current session (if any). Returns undefined if there is no active session.
    // ///
    // /// Pass options.withBalance = true to include the balance of the account.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "getActiveAccountAsync")]
    // pub(crate) fn get_active_account_async(this: &CsprClick, options: &JsValue) -> Result<Promise, JsValue>;

    // /// Gets the public key for the current session (if any). Or undefined if no active session.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "getActivePublicKey")]
    // pub(crate) fn get_active_public_key(this: &CsprClick) -> Result<Promise, JsValue>;

    // /// Returns a ProviderInfo object containing the information of the connected wallet, or the specified in the provider argument.
    // /// Keep in mind that some information is only available if the wallet is connected (e.g. Version of Ledger can only be recovered if the hardware device is connected).
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "getProviderInfo")]
    // pub(crate) fn get_provider_info(this: &CsprClick, provider: Option<String>) -> Result<Promise, JsValue>;

    // /// Returns an object with a list of providers enabled to use in the application and a list of known accounts that can be used to sign in automatically with signInWithAccount().
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "getSignInOptions")]
    // pub(crate) fn get_sign_in_options(this: &CsprClick, refresh: bool) -> Result<Promise, JsValue>;

    // /// Call init to initialize CSPR.click in your web application. This MUST be the first method you call after the downloading of the library.
    // /// See [CsprClickInitOptions](https://docs.cspr.click/cspr.click-sdk/reference/methods#:~:text=See-,CsprClickInitOptions,-for%20reference%20on)
    // /// for reference on the options parameter.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick")]
    // pub(crate) fn init(this: &CsprClick, options: &JsValue) -> Result<(), JsValue>;

    // /// Checks if the provider (not the account) indicated as the first argument is connected to the application.
    // /// Note this check is independent of whether there's an active account on CSPR.click or not or even if that account belongs to the given provider.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "isConnected")]
    // pub(crate) fn is_connected(this: &CsprClick, provider: String) -> Result<Promise, JsValue>;

    // /// Checks if the provider indicated as the first argument is enabled in the application and installed (in case it’s a browser extension).
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "isProviderPresent")]
    // pub(crate) fn is_provider_present(this: &CsprClick, provider: String) -> Result<bool, JsValue>;

    // /// Checks if the provider indicated as the first argument is unlocked.
    // ///
    // /// This method returns undefined when the provider does not offer this information.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "isUnlocked")]
    // pub(crate) fn is_unlocked(this: &CsprClick, provider: String) -> Result<Promise, JsValue>;

    // /// Triggers the mechanisms to request your user to sign a transaction with the active wallet.
    // /// When the user approves the signature, CSPR.click sends the transaction to the Casper network.
    // /// Returns a SendResult object with status information.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "send")]
    // pub(crate) fn send(
    //     this: &CsprClick,
    //     transaction: &JsValue,
    //     signing_public_key: &str,
    //     on_status_update: Option<&js_sys::Function>,
    //     timeout: Option<u32>
    // ) -> Result<Promise, JsValue>;

    // /// Triggers the mechanisms to request your user to sign a transaction with the active wallet.
    // ///
    // /// A SignResult object is returned with the signature value or an error.
    // ///
    // /// The transaction is a json object (or a string) containing either a Deploy or a TransactionV1. In each case, the corresponding wrapper must be present:
    // ///
    // /// Deploy: {"deploy": { "hash": "01....
    // ///
    // /// TransactionV1: {"transaction": {"Version1": {"hash": "01....
    // ///
    // /// signingPublicKey MUST be the public key for the active account. Otherwise, this method will return an error.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick")]
    // pub(crate) fn sign(this: &CsprClick, transaction: &JsValue, signing_public_key: &str) -> Result<Promise, JsValue>;

    // /// Triggers a request to a UI library to show a sign-in dialog.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick")]
    // pub(crate) fn signIn(this: &CsprClick) -> Result<(), JsValue>;

    // /// Starts a session with the indicated account. This account must be one of the accounts returned in getKnownAccounts or getSignInOptions.
    // ///
    // /// Note that no interaction with the account provider is required to sign-in. CSPR.click will check and restore the connection if needed when there's a transaction or message to sign.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "signInWithAccount")]
    // pub(crate) fn sign_in_with_account(this: &CsprClick, account: &AccountType) -> Result<Promise, JsValue>;

    // /// Triggers the mechanisms to request your user to sign a text message with the active wallet.
    // ///
    // /// signingPublicKey MUST be the public key for the active account. Otherwise, this method will return an error.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "signMessage")]
    // pub(crate) fn sign_message(
    //     this: &CsprClick,
    //     message: &str,
    //     signing_public_key: &str
    // ) -> Result<Promise, JsValue>;

    // /// Closes an active session in your dApp.e session in your dApp.
    // ///
    // /// Triggers the csprclick:signed_out event.Triggers the csprclick:signed_out event.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "signOut")]
    // pub(crate) fn sign_out(this: &CsprClick) -> Result<(), JsValue>;

    // /// Call this method to request to the specified wallet to offer the user the selection of a different account. ection of a different account.
    // /// This is valid for providers with its own UI (like browser extenstions).
    // /// Call this method without any provider to request CSPR.click UI to show the Switch Account modal window.
    // #[wasm_bindgen(method, catch, js_class = "CsprClick", js_name = "switchAccount")]
    // pub(crate) fn switch_account(this: &CsprClick, with_provider: Option<String>, options: &JsValue) -> Result<Promise, JsValue>;
}
