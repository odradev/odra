use odra_wasm_client::wasm_bindgen;
use odra_wasm_client::wasm_bindgen_futures;
use odra_wasm_client::JsValueSerdeExt;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub struct StyksPriceFeedWasmClient {
    wasm_client: odra_wasm_client::OdraWasmClient,
    wallet: odra_wasm_client::CasperWallet,
    address: odra_wasm_client::types::Address
}
#[wasm_bindgen]
impl StyksPriceFeedWasmClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "wasmClient")] wasm_client: odra_wasm_client::OdraWasmClient,
        address: odra_wasm_client::types::Address
    ) -> Self {
        StyksPriceFeedWasmClient {
            wasm_client,
            wallet: odra_wasm_client::CasperWallet::default(),
            address
        }
    }
    #[wasm_bindgen(js_name = "setConfig")]
    pub async fn set_config(
        &self,
        #[wasm_bindgen(js_name = "config")] config: StyksPriceFeedConfig
    ) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError>
    {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(odra_wasm_client::wasm_bindgen::JsError::new(
                "Could not connect to the wallet"
            ));
        }
        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "set_config",
                casper_types::runtime_args! { stringify ! (config) => config }
            )
            .await
    }
    #[wasm_bindgen(js_name = "getConfig")]
    pub async fn get_config(
        &self
    ) -> Result<StyksPriceFeedConfig, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_config",
                casper_types::runtime_args! {}
            )
            .await?;
        let result = <StyksPriceFeedConfig as casper_types::bytesrepr::FromBytes>::from_bytes(
            &cl_value.inner_bytes()[4..]
        )
        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }
    #[wasm_bindgen(js_name = "getConfigOrNone")]
    pub async fn get_config_or_none(
        &self
    ) -> Result<Option<StyksPriceFeedConfig>, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_config_or_none",
                casper_types::runtime_args! {}
            )
            .await?;
        let result =
            <Option<StyksPriceFeedConfig> as casper_types::bytesrepr::FromBytes>::from_bytes(
                &cl_value.inner_bytes()[4..]
            )
            .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }
    #[wasm_bindgen(js_name = "getCurrentTwapStore")]
    pub async fn get_current_twap_store(
        &self,
        #[wasm_bindgen(js_name = "id")] id: String
    ) -> Result<Vec<JsValue>, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_current_twap_store",
                casper_types::runtime_args! { stringify ! (id) => id }
            )
            .await?;
        let result = <Vec<Option<u64>> as casper_types::bytesrepr::FromBytes>::from_bytes(
            &cl_value.inner_bytes()[4..]
        )
        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        result
            .0
            .into_iter()
            .map(|v| JsValue::from_serde(&v))
            .collect::<Result<Vec<JsValue>, _>>()
            .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))
    }
    #[wasm_bindgen(js_name = "getLastHeartbeat")]
    pub async fn get_last_heartbeat(
        &self
    ) -> Result<Option<u64>, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_last_heartbeat",
                casper_types::runtime_args! {}
            )
            .await?;
        let result = <Option<u64> as casper_types::bytesrepr::FromBytes>::from_bytes(
            &cl_value.inner_bytes()[4..]
        )
        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }
    #[wasm_bindgen(js_name = "addToFeed")]
    pub async fn add_to_feed(
        &self,
        #[wasm_bindgen(js_name = "input")] input: Vec<JsValue>
    ) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError>
    {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(odra_wasm_client::wasm_bindgen::JsError::new(
                "Could not connect to the wallet"
            ));
        }
        let input: Vec<(String, u64)> = input
            .into_iter()
            .map(|js_value| {
                js_value.into_serde().map_err(|err| {
                    odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err))
                })
            })
            .collect::<Result<_, _>>()?;
        self.wasm_client
            .call_entry_point(
                &self.wallet,
                *self.address,
                "add_to_feed",
                casper_types::runtime_args! { stringify ! (input) => input }
            )
            .await
    }
    #[wasm_bindgen(js_name = "getTwapPrice")]
    pub async fn get_twap_price(
        &self,
        #[wasm_bindgen(js_name = "id")] id: String
    ) -> Result<Option<u64>, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_twap_price",
                casper_types::runtime_args! { stringify ! (id) => id }
            )
            .await?;
        let result = <Option<u64> as casper_types::bytesrepr::FromBytes>::from_bytes(
            &cl_value.inner_bytes()[4..]
        )
        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        Ok(result.0)
    }
    #[wasm_bindgen(js_name = "hasRole")]
    pub async fn has_role(
        &self,
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address
    ) -> Result<bool, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self . wasm_client . call_entry_point_with_proxy (* self . address , "has_role" , casper_types :: runtime_args ! { stringify ! (role) => role , stringify ! (address) => * address }) . await ? ;
        let result =
            <bool as casper_types::bytesrepr::FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
                .map_err(|err| {
                    odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err))
                })?;
        Ok(result.0)
    }
    #[wasm_bindgen(js_name = "grantRole")]
    pub async fn grant_role(
        &self,
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address
    ) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError>
    {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(odra_wasm_client::wasm_bindgen::JsError::new(
                "Could not connect to the wallet"
            ));
        }
        self . wasm_client . call_entry_point (& self . wallet , * self . address , "grant_role" , casper_types :: runtime_args ! { stringify ! (role) => role , stringify ! (address) => * address }) . await
    }
    #[wasm_bindgen(js_name = "revokeRole")]
    pub async fn revoke_role(
        &self,
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address
    ) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError>
    {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(odra_wasm_client::wasm_bindgen::JsError::new(
                "Could not connect to the wallet"
            ));
        }
        self . wasm_client . call_entry_point (& self . wallet , * self . address , "revoke_role" , casper_types :: runtime_args ! { stringify ! (role) => role , stringify ! (address) => * address }) . await
    }
    #[wasm_bindgen(js_name = "getRoleAdmin")]
    pub async fn get_role_admin(
        &self,
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>
    ) -> Result<Vec<u8>, odra_wasm_client::wasm_bindgen::JsError> {
        let cl_value = self
            .wasm_client
            .call_entry_point_with_proxy(
                *self.address,
                "get_role_admin",
                casper_types::runtime_args! { stringify ! (role) => role }
            )
            .await?;
        let result = <Vec<u8> as casper_types::bytesrepr::FromBytes>::from_bytes(
            &cl_value.inner_bytes()[4..]
        )
        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
        Ok(result.0.to_vec())
    }
    #[wasm_bindgen(js_name = "renounceRole")]
    pub async fn renounce_role(
        &self,
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address
    ) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError>
    {
        if !self.wallet.request_connection().await.is_ok() {
            return Err(odra_wasm_client::wasm_bindgen::JsError::new(
                "Could not connect to the wallet"
            ));
        }
        self . wasm_client . call_entry_point (& self . wallet , * self . address , "renounce_role" , casper_types :: runtime_args ! { stringify ! (role) => role , stringify ! (address) => * address }) . await
    }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct RoleAdminChanged {
    #[wasm_bindgen(js_name = "role")]
    pub role: Vec<u8>,
    #[wasm_bindgen(js_name = "previousAdminRole")]
    pub previous_admin_role: Vec<u8>,
    #[wasm_bindgen(js_name = "newAdminRole")]
    pub new_admin_role: Vec<u8>
}
#[wasm_bindgen]
impl RoleAdminChanged {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "previousAdminRole")] previous_admin_role: Vec<u8>,
        #[wasm_bindgen(js_name = "newAdminRole")] new_admin_role: Vec<u8>
    ) -> Self {
        Self {
            role,
            previous_admin_role,
            new_admin_role
        }
    }
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
}
impl casper_types::bytesrepr::FromBytes for RoleAdminChanged {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (role, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (previous_admin_role, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (new_admin_role, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((
            Self {
                role,
                previous_admin_role,
                new_admin_role
            },
            bytes
        ))
    }
}
impl casper_types::bytesrepr::ToBytes for RoleAdminChanged {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.role)?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.previous_admin_role
        )?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.new_admin_role
        )?);
        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        let mut result = 0;
        result += self.role.serialized_length();
        result += self.previous_admin_role.serialized_length();
        result += self.new_admin_role.serialized_length();
        result
    }
}
impl casper_types::CLTyped for RoleAdminChanged {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct RoleGranted {
    #[wasm_bindgen(js_name = "role")]
    pub role: Vec<u8>,
    #[wasm_bindgen(js_name = "address")]
    address: odra_core::prelude::Address,
    #[wasm_bindgen(js_name = "sender")]
    sender: odra_core::prelude::Address
}
#[wasm_bindgen]
impl RoleGranted {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address,
        #[wasm_bindgen(js_name = "sender")] sender: odra_wasm_client::types::Address
    ) -> Self {
        Self {
            role,
            address: address.into(),
            sender: sender.into()
        }
    }
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
    #[wasm_bindgen(setter)]
    pub fn set_address(&mut self, value: odra_wasm_client::types::Address) {
        self.address = value.into();
    }
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> odra_wasm_client::types::Address {
        self.address.into()
    }
    #[wasm_bindgen(setter)]
    pub fn set_sender(&mut self, value: odra_wasm_client::types::Address) {
        self.sender = value.into();
    }
    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> odra_wasm_client::types::Address {
        self.sender.into()
    }
}
impl casper_types::bytesrepr::FromBytes for RoleGranted {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (role, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (address, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (sender, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((
            Self {
                role,
                address,
                sender
            },
            bytes
        ))
    }
}
impl casper_types::bytesrepr::ToBytes for RoleGranted {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.role)?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.address)?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.sender)?);
        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        let mut result = 0;
        result += self.role.serialized_length();
        result += self.address.serialized_length();
        result += self.sender.serialized_length();
        result
    }
}
impl casper_types::CLTyped for RoleGranted {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct RoleRevoked {
    #[wasm_bindgen(js_name = "role")]
    pub role: Vec<u8>,
    #[wasm_bindgen(js_name = "address")]
    address: odra_core::prelude::Address,
    #[wasm_bindgen(js_name = "sender")]
    sender: odra_core::prelude::Address
}
#[wasm_bindgen]
impl RoleRevoked {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "role")] role: Vec<u8>,
        #[wasm_bindgen(js_name = "address")] address: odra_wasm_client::types::Address,
        #[wasm_bindgen(js_name = "sender")] sender: odra_wasm_client::types::Address
    ) -> Self {
        Self {
            role,
            address: address.into(),
            sender: sender.into()
        }
    }
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
    #[wasm_bindgen(setter)]
    pub fn set_address(&mut self, value: odra_wasm_client::types::Address) {
        self.address = value.into();
    }
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> odra_wasm_client::types::Address {
        self.address.into()
    }
    #[wasm_bindgen(setter)]
    pub fn set_sender(&mut self, value: odra_wasm_client::types::Address) {
        self.sender = value.into();
    }
    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> odra_wasm_client::types::Address {
        self.sender.into()
    }
}
impl casper_types::bytesrepr::FromBytes for RoleRevoked {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (role, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (address, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (sender, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((
            Self {
                role,
                address,
                sender
            },
            bytes
        ))
    }
}
impl casper_types::bytesrepr::ToBytes for RoleRevoked {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.role)?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.address)?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(&self.sender)?);
        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        let mut result = 0;
        result += self.role.serialized_length();
        result += self.address.serialized_length();
        result += self.sender.serialized_length();
        result
    }
}
impl casper_types::CLTyped for RoleRevoked {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}
#[derive(Debug, Clone, serde :: Serialize, serde :: Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct StyksPriceFeedConfig {
    #[wasm_bindgen(js_name = "heartbeatInterval")]
    pub heartbeat_interval: u64,
    #[wasm_bindgen(js_name = "heartbeatTolerance")]
    pub heartbeat_tolerance: u64,
    #[wasm_bindgen(js_name = "twapWindow")]
    pub twap_window: u32,
    #[wasm_bindgen(js_name = "twapTolerance")]
    pub twap_tolerance: u32,
    #[wasm_bindgen(js_name = "priceFeedIds")]
    pub price_feed_ids: Vec<String>
}
#[wasm_bindgen]
impl StyksPriceFeedConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(js_name = "heartbeatInterval")] heartbeat_interval: u64,
        #[wasm_bindgen(js_name = "heartbeatTolerance")] heartbeat_tolerance: u64,
        #[wasm_bindgen(js_name = "twapWindow")] twap_window: u32,
        #[wasm_bindgen(js_name = "twapTolerance")] twap_tolerance: u32,
        #[wasm_bindgen(js_name = "priceFeedIds")] price_feed_ids: Vec<String>
    ) -> Self {
        Self {
            heartbeat_interval,
            heartbeat_tolerance,
            twap_window,
            twap_tolerance,
            price_feed_ids
        }
    }
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
}
impl casper_types::bytesrepr::FromBytes for StyksPriceFeedConfig {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        let (heartbeat_interval, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (heartbeat_tolerance, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (twap_window, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (twap_tolerance, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (price_feed_ids, bytes) = casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((
            Self {
                heartbeat_interval,
                heartbeat_tolerance,
                twap_window,
                twap_tolerance,
                price_feed_ids
            },
            bytes
        ))
    }
}
impl casper_types::bytesrepr::ToBytes for StyksPriceFeedConfig {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.heartbeat_interval
        )?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.heartbeat_tolerance
        )?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.twap_window
        )?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.twap_tolerance
        )?);
        result.extend(casper_types::bytesrepr::ToBytes::to_bytes(
            &self.price_feed_ids
        )?);
        Ok(result)
    }
    fn serialized_length(&self) -> usize {
        let mut result = 0;
        result += self.heartbeat_interval.serialized_length();
        result += self.heartbeat_tolerance.serialized_length();
        result += self.twap_window.serialized_length();
        result += self.twap_tolerance.serialized_length();
        result += self.price_feed_ids.serialized_length();
        result
    }
}
impl casper_types::CLTyped for StyksPriceFeedConfig {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::Any
    }
}
