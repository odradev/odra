use casper_types::{bytesrepr::FromBytes, runtime_args};
use wasm_bindgen::prelude::*;

use crate::{
    client::OdraWasmClient,
    types::{address::Address, bigint::U256, transaction::TransactionHash as JsTransactionHash},
    wallet::CasperWallet
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
        &mut self,
        spender: Address,
        amount: U256,
        gas: Option<u64>
    ) -> Result<JsTransactionHash, JsError> {
        if let Some(gas) = gas {
            self.wasm_client.set_gas(gas);
        }
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "approve",
                runtime_args! {
                    "spender" => *spender,
                    "amount" => *amount
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn decimals(&self) -> Result<u8, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(&self.wallet, *self.address, "decimals", runtime_args! {})
            .await?;

        let result = <u8 as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }

    #[wasm_bindgen]
    pub async fn name(&self) -> Result<String, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(&self.wallet, *self.address, "name", runtime_args! {})
            .await?;

        let result = <String as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }
}
