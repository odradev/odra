use std::collections::HashMap;

use casper_types::{CLType, Transaction};
use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::types::{Address, U512};

const USER_ERR_PREFIX: &str = "User error: ";

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignResult {
    /// true when the user has declined the signature of the transaction. false otherwise
    #[wasm_bindgen(readonly, js_name = "isCancelled")]
    #[serde(rename = "cancelled")]
    pub is_cancelled: bool,
    /// An hexadecimal string with the crytpographic signature of the deploy.
    #[wasm_bindgen(readonly, js_name = "signatureHex")]
    #[serde(rename = "signatureHex")]
    pub signature_hex: Option<String>,
    /// A byte array with the cryptographic signature of the deploy.
    #[wasm_bindgen(readonly)]
    pub signature: Vec<u8>,
    transaction: Option<Transaction>,
    /// None if the deploy has been successfully signed. It contains an error message otherwise.
    #[wasm_bindgen(readonly)]
    pub error: Option<String>
}

#[wasm_bindgen]
impl SignResult {
    #[wasm_bindgen(getter)]
    pub fn transaction(&self) -> JsValue {
        if let Some(ref tx) = self.transaction {
            JsValue::from_serde(tx).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct AccountInfo {
    /// The provider to which the account belongs to.
    #[wasm_bindgen(readonly)]
    pub provider: String,
    /// An array of supported capabilities in the connected wallet.
    /// Possible values: "sign-deploy", "sign-transactionv1", "sign-message".
    #[wasm_bindgen(readonly, js_name = "providerSupports")]
    #[serde(rename = "providerSupports")]
    pub provider_supports: Option<Vec<String>>,
    /// CSPR.name name
    #[wasm_bindgen(readonly, js_name = "csprName")]
    pub cspr_name: Option<String>,
    /// The account public key in hex format.
    #[wasm_bindgen(readonly, js_name = "publicKey")]
    pub public_key: String,
    /// Timestamp for the initial connection of the account
    #[wasm_bindgen(readonly, js_name = "connectedAt")]
    pub connected_at: i64,
    /// n/a (for future use)
    #[wasm_bindgen(readonly)]
    token: Option<serde_json::Value>,
    /// Custom data. Depends on the provider.
    custom: Option<serde_json::Value>,
    #[wasm_bindgen(readonly)]
    balance: Option<String>,
    #[wasm_bindgen(readonly, js_name = "liquidBalance")]
    liquid_balance: Option<String>,
    /// URL to the account avatar/logo.
    #[wasm_bindgen(readonly)]
    pub logo: Option<String>
}

#[wasm_bindgen]
impl AccountInfo {
    /// The account address derived from the public key.
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Result<Address, JsError> {
        Address::from_public_key(&self.public_key)
    }

    /// Total balance of the account in CSPR motes (includes liquid +staked balance).
    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> U512 {
        self.balance.as_deref().map(Into::into).unwrap_or_default()
    }

    /// Liquid balance of the account in CSPR motes (includes liquid +staked balance)
    #[wasm_bindgen(getter, js_name = "liquidBalance")]
    pub fn liquid_balance(&self) -> U512 {
        self.liquid_balance
            .as_deref()
            .map(Into::into)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WrappedAccountInfo {
    pub account: AccountInfo
}

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq, Eq)]
#[wasm_bindgen]
#[allow(clippy::upper_case_acronyms)]
pub enum TransactionStatus {
    /// The transaction has been signed and successfully deployed to a Casper node.
    SENT,
    /// The transaction has been processed by the network. May result in success or failure.
    PROCESSED,
    /// The transaction’s time-to-live (TTL) elapsed before execution.
    EXPIRED,
    /// The user rejected the signature request.
    CANCELLED,
    /// The SDK stopped listening for updates before the transaction was finalized. A custom timeout can be specified (default: 120 seconds).
    TIMEOUT,
    /// An unexpected error occurred while submitting or monitoring the transaction.
    ERROR,
    /// A heartbeat event sent periodically to indicate that the connection is still active.
    PING
}

impl<'de> serde::Deserialize<'de> for TransactionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        TransactionStatus::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl TransactionStatus {
    pub fn from_str(status: &str) -> Result<Self, String> {
        match status {
            "sent" => Ok(TransactionStatus::SENT),
            "processed" => Ok(TransactionStatus::PROCESSED),
            "expired" => Ok(TransactionStatus::EXPIRED),
            "cancelled" => Ok(TransactionStatus::CANCELLED),
            "timeout" => Ok(TransactionStatus::TIMEOUT),
            "error" => Ok(TransactionStatus::ERROR),
            "ping" => Ok(TransactionStatus::PING),
            _ => Err(format!("Unknown transaction status: {}", status))
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct TransactionResult {
    /// Transaction status (pending, expired, or processed).
    #[wasm_bindgen(readonly)]
    pub status: Option<TransactionStatus>,
    /// true when the user has declined the signature of the transaction. false otherwise
    #[wasm_bindgen(readonly, js_name = "isCancelled")]
    #[serde(rename = "cancelled")]
    pub is_cancelled: bool,
    #[serde(rename = "deployHash")]
    deploy_hash: Option<String>,
    /// None if the deploy has been successfully executed. It contains an error message otherwise.
    #[wasm_bindgen(readonly)]
    pub error: Option<String>,
    /// Error code extracted from the error message, if available.
    #[wasm_bindgen(readonly, js_name = "errorCode")]
    pub error_code: Option<u16>,
    #[wasm_bindgen(readonly, js_name = "errorData")]
    #[serde(rename = "errorData")]
    error_data: Value,
    #[serde(rename = "transactionHash")]
    transaction_hash: Option<String>,
    /// Transaction details from CSPR Cloud API. null if the transaction is not found or not processed yet.
    #[wasm_bindgen(readonly)]
    #[serde(rename = "csprCloudTransaction")]
    pub data: Option<TransactionData>
}

#[wasm_bindgen]
impl TransactionResult {
    /// If transactionHash is null, returns deployHash. Otherwise, returns transactionHash.
    #[wasm_bindgen(getter, js_name = txHash)]
    pub fn transaction_hash(&self) -> Option<String> {
        if self.transaction_hash.is_none() {
            self.deploy_hash.clone()
        } else {
            self.transaction_hash.clone()
        }
    }
}

impl<'de> serde::Deserialize<'de> for TransactionResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct TransactionResultVisitor;

        impl<'de> Visitor<'de> for TransactionResultVisitor {
            type Value = TransactionResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a TransactionResult object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>
            {
                let mut status: Option<TransactionStatus> = None;
                let mut is_cancelled: Option<bool> = None;
                let mut deploy_hash: Option<String> = None;
                let mut error: Option<String> = None;
                let mut error_data: Option<Value> = None;
                let mut transaction_hash: Option<String> = None;
                let mut data: Option<TransactionData> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "status" => {
                            status = map.next_value()?;
                        }
                        "cancelled" => {
                            is_cancelled = Some(map.next_value()?);
                        }
                        "deployHash" => {
                            deploy_hash = map.next_value()?;
                        }
                        "error" => {
                            error = map.next_value()?;
                        }
                        "errorData" => {
                            error_data = map.next_value()?;
                        }
                        "transactionHash" => {
                            transaction_hash = map.next_value()?;
                        }
                        "csprCloudTransaction" => {
                            data = map.next_value()?;
                        }
                        _ => {
                            // Ignore unknown fields
                            let _ = map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                let error_data = error_data.unwrap_or(Value::Null);
                let is_cancelled =
                    is_cancelled.ok_or_else(|| serde::de::Error::missing_field("cancelled"))?;

                let error_code = if let Some(ref error_str) = error {
                    if error_str == "Out of gas error" {
                        Some(odra_core::prelude::ExecutionError::OutOfGas.code())
                    } else if let Some(stripped) = error_str.strip_prefix(USER_ERR_PREFIX) {
                        stripped.parse().ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(TransactionResult {
                    status,
                    is_cancelled,
                    deploy_hash,
                    error,
                    error_data,
                    transaction_hash,
                    data,
                    error_code
                })
            }
        }

        deserializer.deserialize_map(TransactionResultVisitor)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionData {
    /// Transaction arguments provided for contract execution.
    args: HashMap<String, ArgValue>,
    /// Hash of the block containing the transaction represented as a hexadecimal string.
    #[wasm_bindgen(readonly, js_name = "blockHash")]
    pub block_hash: String,
    /// Height of the block containing the transaction.
    #[wasm_bindgen(readonly, js_name = "blockHeight")]
    pub block_height: u64,
    /// Hash of the transaction caller account represented as a hexademical string.
    #[wasm_bindgen(readonly, js_name = "callerHash")]
    pub caller_hash: String,
    /// Public key of the transaction caller account represented as a hexademical string.
    /// May be null if the public key is not known, but the callerHash will still be present.
    #[wasm_bindgen(readonly, js_name = "callerPublicKey")]
    pub caller_public_key: Option<String>,
    /// Represents the total amount of gas consumed during the execution of the transaction.
    #[wasm_bindgen(readonly, js_name = "consumedGas")]
    pub consumed_gas: String,
    /// Hash of the contract called by the transaction represented as a hexadecimal string.
    /// null if the transaction had no contract call.
    #[wasm_bindgen(readonly, js_name = "contractHash")]
    pub contract_hash: String,
    /// Hash of the contract package called by the transaction represented as a hexadecimal string.
    /// null if the transaction had no contract call.
    #[wasm_bindgen(readonly, js_name = "contractPackageHash")]
    pub contract_package_hash: String,
    /// Transaction execution cost. The type is string to avoid overflow in languages that don't support uint64,
    /// which is the correct type.
    #[wasm_bindgen(readonly)]
    pub cost: String,
    /// Transaction hash represented as a hexadecimal string. Primary transaction identifier.
    #[wasm_bindgen(readonly, js_name = "transactionHash")]
    pub deploy_hash: String,
    /// Identifier of the ContractEntrypoint called by transaction. null if the transaction had no contract call.
    #[wasm_bindgen(readonly, js_name = "entryPointId")]
    pub entry_point_id: u64,
    /// Error message in case of a failed transaction. null for a successful transaction.
    #[wasm_bindgen(readonly, js_name = "errorMessage")]
    pub error_message: Option<String>,
    /// Identifier, that tells what type of the transaction was executed
    #[wasm_bindgen(readonly, js_name = "executionTypeId")]
    pub execution_type_id: u64,
    /// Maximum allowed gas price that was specified by the caller (used only in Limited(0) pricing mode)
    #[wasm_bindgen(readonly, js_name = "gasPriceLimit")]
    pub gas_price_limit: u64,
    /// Indicates whether the transaction uses the standard payment mechanism or a custom payment contract
    #[wasm_bindgen(readonly, js_name = "isStandardPayment")]
    pub is_standard_payment: bool,
    /// Payment amount provided by the caller in motes. The type is string to avoid overflow in languages
    /// that don't support uint64, which is the correct type. null if a custom payment contract was provided
    /// to the transaction instead of the value in motes.
    #[wasm_bindgen(readonly, js_name = "paymentAmount")]
    pub payment_amount: String,
    /// Pricing mode identifier. Indicates which pricing model applies to the transaction
    #[wasm_bindgen(readonly, js_name = "pricingModeId")]
    pub pricing_mode_id: u64,
    /// The amount of gas cost that was refunded to the caller account. In the current Mainnet configuration,
    /// 75% of unused payment amount is refunded.
    #[wasm_bindgen(readonly, js_name = "refundAmount")]
    pub refund_amount: String,
    /// Identifies how the transaction was executed: 0 for native execution, 1 for VM version 1, and 2 for VM version 2
    #[wasm_bindgen(readonly, js_name = "runtimeTypeId")]
    pub runtime_type_id: u64,
    /// Transaction status (pending, expired, or processed).
    #[wasm_bindgen(readonly)]
    pub status: TransactionStatus,
    /// Transaction creation timestamp in the ISO 8601 format.
    #[wasm_bindgen(readonly)]
    pub timestamp: String,
    /// Transaction version identifier: 0 for Casper 1.X transactions, 1 for Casper 2.0 transactions, and 2 for Casper 2.0 transactions.
    #[wasm_bindgen(readonly, js_name = "versionId")]
    pub version_id: u64
}

#[wasm_bindgen]
impl TransactionData {
    #[wasm_bindgen(getter)]
    pub fn args(&self) -> JsValue {
        JsValue::from_serde(&self.args).unwrap_or(JsValue::NULL)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TransactionData {
    pub fn args_map(&self) -> HashMap<String, ArgValue> {
        self.args.clone()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArgValue {
    cl_type: CLType,
    parsed: Option<Value>
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSACTION_RESULT_TEST_DATA: &str = r#"
{
    "cancelled": false,
    "transactionHash": "94429811f595902bb55e1b132a1228e58f831023f2b8d6f4c48919c7d3e51f23",
    "error": null,
    "errorData": null,
    "deployHash": null,
    "status": "processed",
    "csprCloudTransaction": {
        "deploy_hash": "94429811f595902bb55e1b132a1228e58f831023f2b8d6f4c48919c7d3e51f23",
        "block_hash": "4fd53aa64f6b37e342a848913dfda6f29b443cdd724a8efd5c478c850fc2a5e3",
        "block_height": 5714663,
        "caller_public_key": null,
        "caller_hash": "5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364",
        "execution_type_id": 4,
        "contract_package_hash": "8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675",
        "contract_hash": "575bd3677220fe8ff1915c588bda07fd7959671b773bb10ae1f387ca86d41789",
        "entry_point_id": 2658797,
        "args": {
            "amount": {
                "cl_type": "U256",
                "parsed": "1000000000"
            },
            "id": {
                "cl_type": {
                    "Option": "U64"
                },
                "parsed": null
            },
            "target": {
                "cl_type": {
                    "ByteArray": 32
                },
                "parsed": "536345751b7c6c6299d5ef10862d76736ed062bc32c1dabcd1179c06469d93ca"
            }
        },
        "payment_amount": "3000000000",
        "refund_amount": "38057400",
        "version_id": 2,
        "pricing_mode_id": 0,
        "gas_price_limit": 5,
        "is_standard_payment": true,
        "runtime_type_id": 1,
        "cost": "3000000000",
        "consumed_gas": "2949256800",
        "error_message": null,
        "status": "processed",
        "timestamp": "2025-10-06T06:13:50.815Z"
    }
}
"#;

    #[test]
    fn test_deserialize_transaction_result() {
        let result: TransactionResult =
            serde_json::from_str(TRANSACTION_RESULT_TEST_DATA).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::PROCESSED));
        assert!(!result.is_cancelled);
        assert_eq!(
            result.transaction_hash.as_deref(),
            Some("94429811f595902bb55e1b132a1228e58f831023f2b8d6f4c48919c7d3e51f23")
        );
        assert!(result.error.is_none());
        assert!(result.error_data.is_null());
        assert!(result.data.is_some());
        assert_eq!(result.error_code, None); // No error, so error_code should be None
        let data = result.data.unwrap();
        assert_eq!(data.block_height, 5714663);
        assert_eq!(data.args.get("amount").unwrap().cl_type, CLType::U256);
    }

    #[test]
    fn test_deserialize_transaction_result_with_out_of_gas_error() {
        let test_data = r#"
{
    "cancelled": false,
    "transactionHash": null,
    "error": "Out of gas error",
    "errorData": null,
    "deployHash": null,
    "status": "error",
    "csprCloudTransaction": null
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::ERROR));
        assert_eq!(result.error.as_deref(), Some("Out of gas error"));
        assert_eq!(
            result.error_code,
            Some(odra_core::prelude::ExecutionError::OutOfGas.code())
        );
    }

    #[test]
    fn test_deserialize_transaction_result_with_user_error() {
        let test_data = r#"
{
    "cancelled": false,
    "transactionHash": null,
    "error": "User error: 65001",
    "errorData": null,
    "deployHash": null,
    "status": "error",
    "csprCloudTransaction": null
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::ERROR));
        assert_eq!(result.error.as_deref(), Some("User error: 65001"));
        assert_eq!(result.error_code, Some(65001));
    }

    #[test]
    fn test_deserialize_transaction_result_with_other_error() {
        let test_data = r#"
{
    "cancelled": false,
    "transactionHash": null,
    "error": "Some other error",
    "errorData": null,
    "deployHash": null,
    "status": "error",
    "csprCloudTransaction": null
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::ERROR));
        assert_eq!(result.error.as_deref(), Some("Some other error"));
        assert_eq!(result.error_code, None); // Unknown error types default to None
    }

    #[test]
    fn test_null_status_and_error_data() {
        let test_data = r#"
{
    "cancelled": false,
    "transactionHash": "67b6c0fcbdf2d5ece86ca90dff3b64f30b9c3a3cb4a80a24ae4ee862f0aa893d",
    "error": null,
    "errorData": null,
    "deployHash": null,
    "status": null,
    "csprCloudTransaction": null
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, None);
        assert!(!result.is_cancelled);
        assert_eq!(
            result.transaction_hash.as_deref(),
            Some("67b6c0fcbdf2d5ece86ca90dff3b64f30b9c3a3cb4a80a24ae4ee862f0aa893d")
        );
        assert!(result.error.is_none());
        assert_eq!(result.error_data, Value::Null);
        assert!(result.data.is_none());
        assert_eq!(result.error_code, None); // No error, so error_code should be None
    }

    #[test]
    fn test_deserialize_transaction_result_with_error_data() {
        let test_data = r#"
{
        "cancelled":false,
        "deployHash":null,
        "transactionHash":null,
        "error":"Code: -32016, err: Invalid transaction",
        "errorData":{
            "code":-32016,
            "message":"Invalid transaction",
            "data":"the transaction was invalid: The transaction sent to the network had an invalid chain name"
        },
        "status":"error",
        "csprCloudTransaction":null
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::ERROR));
        assert_eq!(
            result.error.as_deref(),
            Some("Code: -32016, err: Invalid transaction")
        );
        assert_eq!(
            result.error_data,
            serde_json::json!({
                "code": -32016,
                "message": "Invalid transaction",
                "data": "the transaction was invalid: The transaction sent to the network had an invalid chain name"
            })
        );
    }

    #[test]
    fn test_list_u8_arg() {
        let test_data = r#"
{
        "cancelled":false,
        "transactionHash":"ee9e94fe2ad99fd32977dfa1c90ff86d324ff3bdecb773c805ab0ce3108e14ff",
        "error":null,
        "errorData":null,
        "deployHash":null,
        "status":"processed",
        "csprCloudTransaction":{
            "deploy_hash":"ee9e94fe2ad99fd32977dfa1c90ff86d324ff3bdecb773c805ab0ce3108e14ff",
            "block_hash":"ac19271d0dcc940474867a8625fe6756ebf6b592f5bd58907710e10c4304efed",
            "block_height":5725978,
            "caller_public_key":null,
            "caller_hash":"1ef371ec8f3626883a90a68c400672df0e988c0f76e7fd28beedfdb283a05bd4",
            "execution_type_id":7,
            "contract_package_hash":"8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675",
            "contract_hash":"575bd3677220fe8ff1915c588bda07fd7959671b773bb10ae1f387ca86d41789",
            "entry_point_id":2658790,
            "args":{
                "amount":{
                    "cl_type":"U512",
                    "parsed":"2000000000"
                },
                "args": {
                    "cl_type":{"List":"U8"},
                    "parsed":[0,0,0,0]
                },
                "attached_value":{
                    "cl_type":"U512",
                    "parsed":"2000000000"
                },
                "entry_point":{
                    "cl_type":"String",
                    "parsed":"deposit"
                },
                "package_hash":{
                    "cl_type":{"ByteArray":32},
                    "parsed":"8bc2e4b85757651812f01bc65a37d5df221ac5110254a77ad29d07017110a675"}
                },
            "payment_amount":"2500000000",
            "refund_amount":"1152168865",
            "version_id":2,
            "pricing_mode_id":0,
            "gas_price_limit":5,
            "is_standard_payment":true,
            "runtime_type_id":1,
            "cost":"2500000000",
            "consumed_gas":"963774846",
            "error_message":null,
            "status":"processed",
            "timestamp":"2025-10-08T09:39:01.81Z"
        }
}
"#;
        let result: TransactionResult =
            serde_json::from_str(test_data).expect("Deserialization failed");
        assert_eq!(result.status, Some(TransactionStatus::PROCESSED));
        assert!(result.error.is_none());
        assert!(result.error_data.is_null());
        let args = result.data.map(|d| d.args_map()).expect("No args found");
        assert_eq!(args.len(), 5);
    }
}
