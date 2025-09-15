use crate::{
    js::{casper_wallet_provider, CasperWalletProvider},
    types::{Address, Deploy, PublicKey, SignatureResponse, Transaction}
};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub struct CasperWallet {
    provider: CasperWalletProvider
}

impl Default for CasperWallet {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl CasperWallet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        CasperWallet {
            provider: casper_wallet_provider()
        }
    }

    #[deprecated(note = "prefer signTransaction")]
    #[allow(deprecated)]
    #[wasm_bindgen(js_name = "signDeploy")]
    pub async fn sign_deploy(
        &self,
        deploy: Deploy,
        public_key: Option<String>
    ) -> Result<Deploy, JsError> {
        let is_connected = self.request_connection().await.is_ok();

        if !is_connected {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        let public_key = self.get_public_or_active_key(public_key).await?;

        let deploy_json = deploy
            .to_json_string()
            .map_err(|err| JsError::new(&format!("Failed to serialize deploy: {err:?}")))?;

        let sign = JsFuture::from(
            self.provider
                .sign(
                    &format!("{{\"deploy\":{deploy_json}}}"),
                    &public_key.to_string()
                )
                .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?;

        let signature_response: SignatureResponse = sign
            .into_serde()
            .map_err(|err| JsError::new(&format!("Deserialize signature failed: {err:?}")))?;

        if signature_response.is_cancelled() {
            return Err(JsError::new(&format!(
                "Could not sign deploy for key {public_key}"
            )));
        }

        let signature = format!(
            "0{}{}",
            public_key.tag(),
            signature_response.get_signature_hex()
        );
        let signed_deploy = deploy.add_signature(&public_key.to_string(), &signature);
        Ok(signed_deploy)
    }

    #[wasm_bindgen(js_name = "signTransaction")]
    pub async fn sign_transaction(
        &self,
        transaction: Transaction,
        public_key: Option<String>
    ) -> Result<Transaction, JsError> {
        let is_connected = self.request_connection().await.is_ok();

        if !is_connected {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        let public_key = self.get_public_or_active_key(public_key).await?;

        let transaction_json = transaction
            .to_json_string()
            .map_err(|err| JsError::new(&format!("Failed to serialize transaction: {err:?}")))?;

        let sign = JsFuture::from(
            self.provider
                .sign(&transaction_json, &public_key.to_string())
                .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?;

        let signature_response: SignatureResponse = sign
            .into_serde()
            .map_err(|err| JsError::new(&format!("Deserialize signature failed: {err:?}")))?;

        if signature_response.is_cancelled() {
            return Err(JsError::new(&format!(
                "Could not sign transaction for key {public_key}"
            )));
        }

        let signature = format!(
            "0{}{}",
            public_key.tag(),
            signature_response.get_signature_hex()
        );
        let signed_transaction = transaction.add_signature(&public_key.to_string(), &signature);
        Ok(signed_transaction)
    }

    /// Alias for the `sign_message` function, specifically for signing transaction hashes.
    ///
    /// This function calls `sign_message` to sign the provided transaction hash with the
    /// given or active public key.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - The transaction hash string to be signed.
    /// * `public_key` - An optional public key string. If `None`, the active public key is used.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The signature string.
    /// * `Err(JsError)` - An error if the signing process fails.
    #[wasm_bindgen(js_name = "signTransactionHash")]
    pub async fn sign_transaction_hash_js_alias(
        &self,
        transaction_hash: String,
        public_key: Option<String>
    ) -> Result<String, JsError> {
        self.sign_message(transaction_hash, public_key).await
    }

    /// Signs a message with the provided or active public key.
    ///
    /// This function requests a connection to the wallet, retrieves the public key
    /// (either provided or active), signs the message, and returns the signature.
    ///
    /// # Arguments
    ///
    /// * `message` - The message string to be signed.
    /// * `public_key` - An optional public key string. If `None`, the active public key is used.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The signature string.
    /// * `Err(JsError)` - An error if the connection fails, the public key retrieval fails,
    ///   the signing fails, or if the signing is cancelled.
    ///
    /// # Errors
    ///
    /// This function returns a `JsError` if:
    /// * The connection to the wallet could not be established.
    /// * The public key could not be retrieved.
    /// * The signing operation fails.
    /// * The signing is cancelled by the user.
    #[wasm_bindgen(js_name = "signMessage")]
    pub async fn sign_message(
        &self,
        message: String,
        public_key: Option<String>
    ) -> Result<String, JsError> {
        let is_connected = self.request_connection().await.is_ok();
        if !is_connected {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        let public_key = self.get_public_or_active_key(public_key).await?;

        let sign = JsFuture::from(
            self.provider
                .signMessage(&message, &public_key.to_string())
                .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?;

        let signature_response: SignatureResponse = sign
            .into_serde()
            .map_err(|err| JsError::new(&format!("Deserialize signature failed: {err:?}")))?;

        if signature_response.is_cancelled() {
            return Err(JsError::new(&format!(
                "Could not sign deploy for key {public_key}"
            )));
        }
        let signature = format!(
            "0{}{}",
            public_key.tag(),
            signature_response.get_signature_hex()
        );
        Ok(signature)
    }

    #[wasm_bindgen(js_name = "connect")]
    pub async fn request_connection(&self) -> Result<(), JsError> {
        let connection = JsFuture::from(
            self.provider
                .requestConnection()
                .map_err(|err| JsError::new(&format!("Connection failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Connection failed: {err:?}")))?;

        if connection.as_bool().unwrap_or(false) {
            Ok(())
        } else {
            Err(JsError::new("Connection failed"))
        }
    }

    #[wasm_bindgen(js_name = "disconnect")]
    pub async fn disconnect_from_site(&self) -> Result<bool, JsError> {
        let disconnection = JsFuture::from(
            self.provider
                .disconnectFromSite()
                .map_err(|err| JsError::new(&format!("Disconnection failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Disconnection failed: {err:?}")))?;

        if disconnection.as_bool().unwrap_or(false) {
            Ok(true)
        } else {
            Err(JsError::new("Connection failed"))
        }
    }

    #[wasm_bindgen(js_name = "isConnected")]
    pub async fn is_connected(&self) -> Result<bool, JsError> {
        let connection = JsFuture::from(
            self.provider
                .isConnected()
                .map_err(|err| JsError::new(&format!("Connection failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Connection failed: {err:?}")))?;
        Ok(connection.as_bool().unwrap_or_default())
    }

    #[wasm_bindgen(js_name = "getVersion")]
    pub async fn get_version(&self) -> Result<String, JsError> {
        let version = JsFuture::from(
            self.provider
                .getVersion()
                .map_err(|err| JsError::new(&format!("getVersion failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("getVersion failed: {err:?}")))?;

        let version = version
            .as_string()
            .ok_or_else(|| JsError::new("getVersion failed"))?;

        if version.is_empty() {
            return Err(JsError::new("getVersion returned an empty string"));
        }
        Ok(version)
    }

    #[wasm_bindgen(js_name = "getActivePublicKey")]
    pub async fn get_active_public_key(&self) -> Result<String, JsError> {
        let public_key = JsFuture::from(
            self.provider
                .getActivePublicKey()
                .map_err(|err| JsError::new(&format!("getActivePublicKey failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("getActivePublicKey failed: {err:?}")))?;

        let public_key = public_key
            .as_string()
            .ok_or_else(|| JsError::new("getActivePublicKey failed"))?;

        if public_key.is_empty() {
            return Err(JsError::new("getActivePublicKey returned an empty string"));
        }
        Ok(public_key)
    }

    #[wasm_bindgen(js_name = "switchAccount")]
    pub async fn request_switch_account(&self) -> Result<bool, JsError> {
        let switch = JsFuture::from(
            self.provider
                .requestSwitchAccount()
                .map_err(|err| JsError::new(&format!("requestSwitchAccount failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("requestSwitchAccount failed: {err:?}")))?;

        if !switch.as_bool().unwrap_or(false) {
            return Err(JsError::new("requestSwitchAccount failed"));
        }
        Ok(true)
    }

    async fn get_public_or_active_key(
        &self,
        provided_public_key: Option<String>
    ) -> Result<PublicKey, JsError> {
        let public_key = if let Some(public_key) = provided_public_key {
            if !public_key.is_empty() {
                public_key
            } else {
                self.get_active_public_key().await?
            }
        } else {
            self.get_active_public_key().await?
        };

        PublicKey::new(&public_key).map_err(|err| {
            JsError::new(&format!(
                "Failed to create Public key from {public_key}: {err:?}"
            ))
        })
    }

    /// Returns the address of the caller.
    #[wasm_bindgen(js_name = "caller")]
    pub async fn caller(&self) -> Result<Address, JsError> {
        let pk_string = self.get_active_public_key().await?;
        PublicKey::new(&pk_string)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(Into::<Address>::into)
    }
}
