use casper_types::{runtime_args, Timestamp, Transaction};
use wasm_bindgen::prelude::*;

use crate::{
    types::{
        address::Address, bigint::U256, public_key::PublicKey, transaction::{Transaction as JsTransaction, TransactionHash as JsTransactionHash}
    },
    wallet::CasperWallet,
    OdraWasmClient
};

#[wasm_bindgen]
pub struct Cep18Client {
    wasm_client: OdraWasmClient,
    wallet: CasperWallet,
    address: Address
}

#[wasm_bindgen]
impl Cep18Client {
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_client: OdraWasmClient, address: Address) -> Self {
        Cep18Client {
            wasm_client,
            wallet: CasperWallet::default(),
            address
        }
    }

    #[wasm_bindgen]
    pub async fn approve(
        &self,
        spender: Address,
        amount: U256
    ) -> Result<JsTransactionHash, JsError> {
        crate::js::log("starting approval");
        let node_address = self.wasm_client.node_address();
        crate::js::log(&format!("Node address: {node_address}"));
        let caller = self.wallet.get_active_public_key().await?;

        let pk = PublicKey::new(&caller).map_err(|e| JsError::new(&e.to_string()))?;
        let caller = Address::from(pk);
        let transaction: Transaction = self.wasm_client.new_call_transaction(
            *caller,
            *self.address,
            "approve",
            runtime_args! {
                "spender" => *spender,
                "amount" => *amount
            },
            0,
        ).ok_or_else(|| JsError::new("Failed to create transaction"))?;
        crate::js::log("transaction created");

        let signed_transaction: JsTransaction = self
            .wallet
            .sign_transaction(transaction.into(), None)
            .await?;

        crate::js::log("transaction signed");

        self.wasm_client
            .put_transaction(signed_transaction.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e))
    }
}
