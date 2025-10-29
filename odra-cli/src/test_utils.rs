#![allow(unused_variables)]
use odra::{
    casper_types::{
        bytesrepr::{Bytes, ToBytes},
        PublicKey, RuntimeArgs, U512
    },
    entry_point_callback::EntryPointsCaller,
    host::{HostContext, HostEnv},
    prelude::*,
    schema::{
        casper_contract_schema::{Access, Argument, Entrypoint, NamedCLType, Type},
        SchemaCustomTypes
    },
    CallDef, EventError, GasReport
};

use crate::{
    cmd::args::CommandArg,
    container::{ContractError, ContractStorage, ContractsData},
    custom_types::CustomTypeSet,
    DeployedContractsContainer
};

pub fn mock_entry_point() -> Entrypoint {
    Entrypoint {
        name: "test".to_string(),
        description: None,
        is_mutable: false,
        arguments: vec![
            Argument::new(
                "voucher",
                "",
                NamedCLType::Custom("PaymentVoucher".to_string())
            ),
            Argument::new(
                "signature",
                "",
                NamedCLType::List(Box::new(NamedCLType::U8))
            ),
        ],
        return_ty: Type(NamedCLType::Bool),
        is_contract_context: true,
        access: Access::Public
    }
}

pub fn mock_command_args() -> Vec<CommandArg> {
    vec![
        CommandArg::new("voucher.payment.buyer", "", NamedCLType::Key).required(),
        CommandArg::new("voucher.payment.payment_id", "", NamedCLType::String).required(),
        CommandArg::new("voucher.payment.amount", "", NamedCLType::U512).required(),
        CommandArg::new("voucher.names.label", "", NamedCLType::String)
            .required()
            .list(),
        CommandArg::new("voucher.names.owner", "", NamedCLType::Key)
            .required()
            .list(),
        CommandArg::new("voucher.names.token_expiration", "", NamedCLType::U64)
            .required()
            .list(),
        CommandArg::new("voucher.voucher_expiration", "", NamedCLType::U64).required(),
        CommandArg::new(
            "signature",
            "",
            NamedCLType::List(Box::new(NamedCLType::U8))
        )
        .required()
        .list(),
    ]
}

pub fn custom_types() -> CustomTypeSet {
    let mut types = CustomTypeSet::from_iter(PaymentVoucher::schema_types().into_iter().flatten());
    types.extend(NameTokenMetadata::schema_types().into_iter().flatten());
    types
}

#[odra::odra_type]
pub struct NameTokenMetadata {
    pub token_hash: String,
    pub expiration: u64,
    pub resolver: Option<Address>
}

#[odra::odra_type]
pub struct PaymentVoucher {
    pub payment: PaymentInfo,
    pub names: Vec<NameMintInfo>,
    pub voucher_expiration: u64
}

impl PaymentVoucher {
    pub fn new(payment: PaymentInfo, names: Vec<NameMintInfo>, voucher_expiration: u64) -> Self {
        Self {
            payment,
            names,
            voucher_expiration
        }
    }
}

#[odra::odra_type]
pub struct PaymentInfo {
    pub buyer: Address,
    pub payment_id: String,
    pub amount: U512
}

impl PaymentInfo {
    pub fn new(buyer: &str, payment_id: &str, amount: &str) -> Self {
        Self {
            buyer: buyer.parse().unwrap(),
            payment_id: payment_id.to_string(),
            amount: U512::from_dec_str(amount).unwrap()
        }
    }
}

#[odra::odra_type]
pub struct NameMintInfo {
    pub label: String,
    pub owner: Address,
    pub token_expiration: u64
}

impl NameMintInfo {
    pub fn new(label: &str, owner: &str, token_expiration: u64) -> Self {
        Self {
            label: label.to_string(),
            owner: owner.parse().unwrap(),
            token_expiration
        }
    }
}

#[odra::module]
pub struct TestContract;

#[odra::module]
impl TestContract {
    pub fn add(&self, x: u64, y: u64) -> u64 {
        x + y
    }

    pub fn echo(&self, message: String) -> String {
        message
    }

    pub fn mutable(&mut self) {}

    #[allow(clippy::too_many_arguments)]
    pub fn various_args(
        &self,
        a: Address,
        b: Option<Address>,
        c: Vec<String>,
        d: Result<u64, String>,
        e: Result<u64, String>,
        f: (String,),
        g: (String, String),
        h: (String, String, String),
        i: BTreeMap<String, u64>,
        j: Bytes
    ) -> Vec<u8> {
        (a, b, c, d, e).to_bytes().unwrap_or_revert(self)
    }

    #[odra(payable)]
    pub fn deposit(&mut self) {}
}

struct DummyHostCtx;

impl HostContext for DummyHostCtx {
    #[doc = " Sets the caller address for the current contract execution."]
    fn set_caller(&self, caller: Address) {
        todo!()
    }

    #[doc = " Sets the gas limit for the current contract execution."]
    fn set_gas(&self, gas: u64) {
        todo!()
    }

    #[doc = " Returns the caller address for the current contract execution."]
    fn caller(&self) -> Address {
        todo!()
    }

    #[doc = " Returns the account address at the specified index."]
    fn get_account(&self, index: usize) -> Address {
        todo!()
    }

    #[doc = " Returns the validator public key."]
    fn get_validator(&self, index: usize) -> PublicKey {
        todo!()
    }

    #[doc = " The validator at the given index will withdraw all funds and be removed from the validator set."]
    fn remove_validator(&self, index: usize) {
        todo!()
    }

    #[doc = " Returns the CSPR balance of the specified address."]
    fn balance_of(&self, address: &Address) -> U512 {
        todo!()
    }

    #[doc = " Advances the block time by the specified time difference."]
    fn advance_block_time(&self, time_diff: u64) {
        todo!()
    }

    #[doc = " Advances the block time by the specified time difference and processes auctions."]
    fn advance_with_auctions(&self, time_diff: u64) {
        todo!()
    }

    #[doc = " Time between auctions in milliseconds."]
    fn auction_delay(&self) -> u64 {
        todo!()
    }

    #[doc = " Time for the funds to be transferred back to the delegator after undelegation in milliseconds."]
    fn unbonding_delay(&self) -> u64 {
        todo!()
    }

    #[doc = " Returns the delegated amount for the specified delegator and validator."]
    fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        todo!()
    }

    #[doc = " Returns the current block time."]
    fn block_time(&self) -> u64 {
        todo!()
    }

    #[doc = " Returns the event bytes for the specified contract address and index."]
    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        todo!()
    }

    #[doc = " Returns the native event bytes for the specified contract address and index."]
    fn get_native_event(
        &self,
        contract_address: &Address,
        index: u32
    ) -> Result<Bytes, EventError> {
        todo!()
    }

    #[doc = " Returns the number of emitted events for the specified contract address."]
    fn get_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        todo!()
    }

    #[doc = " Returns the number of emitted native events for the specified contract address."]
    fn get_native_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        todo!()
    }

    #[doc = " Calls a contract at the specified address with the given call definition."]
    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        todo!()
    }

    #[doc = " Creates a new contract with the specified name, initialization arguments, and entry points caller."]
    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        todo!()
    }

    fn upgrade_contract(
        &self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        todo!()
    }

    #[doc = " Registers an existing contract with the specified address, name, and entry points caller."]
    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        todo!()
    }

    #[doc = " Returns the contract environment."]
    fn contract_env(&self) -> ContractEnv {
        todo!()
    }

    #[doc = " Returns the gas report for the current contract execution."]
    fn gas_report(&self) -> GasReport {
        todo!()
    }

    #[doc = " Returns the gas cost of the last contract call."]
    fn last_call_gas_cost(&self) -> u64 {
        todo!()
    }

    #[doc = " Signs the specified message with the given address and returns the signature."]
    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        todo!()
    }

    #[doc = " Returns the public key associated with the specified address."]
    fn public_key(&self, address: &Address) -> PublicKey {
        todo!()
    }

    #[doc = " Transfers the specified amount of CSPR from the current caller to the specified address."]
    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        todo!()
    }
}

pub fn mock_host_env() -> HostEnv {
    HostEnv::new(Rc::new(RefCell::new(DummyHostCtx)))
}

struct MockContractStorage;

impl ContractStorage for MockContractStorage {
    fn read(&self) -> Result<ContractsData, ContractError> {
        Ok(ContractsData::default())
    }

    fn write(&mut self, data: &ContractsData) -> Result<(), ContractError> {
        Ok(())
    }

    fn backup(&self) -> Result<(), ContractError> {
        Ok(())
    }
}

pub fn mock_contracts_container() -> DeployedContractsContainer {
    DeployedContractsContainer::instance(MockContractStorage)
}
