use casper_types::runtime_args;
use wasm_bindgen::prelude::*;

use crate::{
    client::OdraWasmClient,
    types::{Address, FromWasmValue, TransactionHash as JsTransactionHash, U256, U512}
};

#[wasm_bindgen]
pub struct WCSPRClient {
    wasm_client: OdraWasmClient,
    address: Address
}

#[wasm_bindgen]
impl WCSPRClient {
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_client: &OdraWasmClient, address: Address) -> Self {
        WCSPRClient {
            wasm_client: wasm_client.clone(),
            address
        }
    }

    #[wasm_bindgen]
    pub async fn decimals(&self) -> Result<u8, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<u8, u8>(*self.address, "decimals", runtime_args! {})
            .await
    }

    #[wasm_bindgen]
    pub async fn name(&self) -> Result<String, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<String, String>(*self.address, "name", runtime_args! {})
            .await
    }

    #[wasm_bindgen]
    pub async fn symbol(&self) -> Result<String, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<String, String>(
                *self.address,
                "symbol",
                runtime_args! {}
            )
            .await
    }

    #[wasm_bindgen(js_name = "totalSupply")]
    pub async fn total_supply(&self) -> Result<U256, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<U256, casper_types::U256>(
                *self.address,
                "total_supply",
                runtime_args! {}
            )
            .await
    }

    #[wasm_bindgen(js_name = "balanceOf")]
    pub async fn balance_of(&self, address: Address) -> Result<U256, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<U256, casper_types::U256>(
                *self.address,
                "balance_of",
                runtime_args! {
                    "address" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(address)?
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<U256, JsError> {
        self.wasm_client
            .call_entry_point_with_proxy::<U256, casper_types::U256>(
                *self.address,
                "allowance",
                runtime_args! {
                    "owner" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(owner)?,
                    "spender" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(spender)?
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn approve(
        &mut self,
        spender: Address,
        amount: U256
    ) -> Result<JsTransactionHash, JsError> {
        self.wasm_client
            .call_entry_point(
                *self.address,
                "approve",
                runtime_args! {
                    "spender" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(spender)?,
                    "amount" => FromWasmValue::<casper_types::U256>::from_wasm_value(amount)?
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn transfer(
        &mut self,
        recipient: Address,
        amount: U256
    ) -> Result<JsTransactionHash, JsError> {
        self.wasm_client
            .call_entry_point(
                *self.address,
                "transfer",
                runtime_args! {
                    "recipient" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(recipient)?,
                    "amount" => FromWasmValue::<casper_types::U256>::from_wasm_value(amount)?
                }
            )
            .await
    }

    #[wasm_bindgen(js_name = "transferFrom")]
    pub async fn transfer_from(
        &mut self,
        owner: Address,
        recipient: Address,
        amount: U256
    ) -> Result<JsTransactionHash, JsError> {
        self.wasm_client
            .call_entry_point(
                *self.address,
                "transfer_from",
                runtime_args! {
                    "owner" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(owner)?,
                    "recipient" => FromWasmValue::<odra_core::prelude::Address>::from_wasm_value(recipient)?,
                    "amount" => FromWasmValue::<casper_types::U256>::from_wasm_value(amount)?
                }
            )
            .await
    }

    #[wasm_bindgen]
    pub async fn deposit(&mut self, attached_value: U512) -> Result<JsTransactionHash, JsError> {
        self.wasm_client
            .call_payable_entry_point(*self.address, "deposit", runtime_args! {}, *attached_value)
            .await
    }

    #[wasm_bindgen]
    pub async fn withdraw(&mut self, amount: U256) -> Result<JsTransactionHash, JsError> {
        self.wasm_client
            .call_entry_point(
                *self.address,
                "withdraw",
                runtime_args! {
                    "amount" => FromWasmValue::<casper_types::U256>::from_wasm_value(amount)?
                }
            )
            .await
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum WCSPRErrors {
    #[doc = "The user cannot target themselves."]
    CannotTargetSelfUser = 60003isize,
    #[doc = "Spender does not have enough allowance approved."]
    InsufficientAllowance = 60002isize,
    #[doc = "Spender does not have enough balance."]
    InsufficientBalance = 60001isize
}
