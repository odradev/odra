use casper_types::{bytesrepr::FromBytes, runtime_args};
use wasm_bindgen::prelude::*;

use crate::{
    client::OdraWasmClient,
    types::{Address, TransactionHash as JsTransactionHash, U256, U512},
    wallet::CasperWallet
};

#[wasm_bindgen]
pub struct WCSPRClient {
    wasm_client: OdraWasmClient,
    wallet: CasperWallet,
    address: Address
}

#[wasm_bindgen]
impl WCSPRClient {
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_client: OdraWasmClient, address: Address) -> Self {
        WCSPRClient {
            wasm_client,
            wallet: CasperWallet::default(),
            address
        }
    }

    #[wasm_bindgen]
    pub async fn decimals(&self) -> Result<u8, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(*self.address, "decimals", runtime_args! {})
            .await?;

        let result = <u8 as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.into())
    }

    #[wasm_bindgen]
    pub async fn name(&self) -> Result<String, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(*self.address, "name", runtime_args! {})
            .await?;

        let result = <String as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.into())
    }

    #[wasm_bindgen]
    pub async fn symbol(&self) -> Result<String, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(*self.address, "symbol", runtime_args! {})
            .await?;

        let result = <String as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }

    #[wasm_bindgen(js_name = "totalSupply")]
    pub async fn total_supply(&self) -> Result<U256, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(*self.address, "total_supply", runtime_args! {})
            .await?;

        let result = <casper_types::U256 as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.into())
    }

    #[wasm_bindgen(js_name = "balanceOf")]
    pub async fn balance_of(&self, address: Address) -> Result<U256, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "balance_of",
                runtime_args! {
                    "owner" => *address
                }
            )
            .await?;

        let result = <casper_types::U256 as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.into())
    }

    #[wasm_bindgen]
    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256, JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "allowance",
                runtime_args! {
                    "owner" => *owner,
                    "spender" => *spender
                }
            )
            .await?;

        let result = <casper_types::U256 as FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
            .map_err(|err| JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.into())
    }

    #[wasm_bindgen]
    pub async fn approve(
        &mut self,
        spender: Address,
        amount: U256,
    ) -> Result<JsTransactionHash, JsError> {
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
    pub async fn transfer(
        &mut self,
        recipient: Address,
        amount: U256,
    ) -> Result<JsTransactionHash, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "transfer",
                runtime_args! {
                    "recipient" => *recipient,
                    "amount" => *amount
                }
            )
            .await
    }

    #[wasm_bindgen(js_name = "transferFrom")]
    pub async fn transfer_from(
        &mut self,
        owner: Address,
        recipient: Address,
        amount: U256,
    ) -> Result<JsTransactionHash, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "transfer_from",
                runtime_args! {
                    "owner" => *owner,
                    "recipient" => *recipient,
                    "amount" => *amount
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn deposit(
        &mut self,
        attached_value: U512
    ) -> Result<JsTransactionHash, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        self.wasm_client
            .call_payable_entry_point(
                &self.wallet,
                *self.address,
                "deposit",
                runtime_args! {},
                *attached_value
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn withdraw(
        &mut self,
        amount: U256,
    ) -> Result<JsTransactionHash, JsError> {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(JsError::new("Could not connect to the wallet"));
        }

        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "burn",
                runtime_args! {
                    "amount" => *amount
                },
            )
            .await
    }
}
