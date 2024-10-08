use odra::schema::{
    casper_contract_schema::{Access, Argument, Entrypoint, NamedCLType, Type},
    SchemaCustomTypes
};
use odra::{casper_types::U512, Address};

use crate::{CommandArg, CustomTypeSet};

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
        CommandArg::new("voucher.payment.buyer", "", NamedCLType::Key, true, false),
        CommandArg::new(
            "voucher.payment.payment_id",
            "",
            NamedCLType::String,
            true,
            false
        ),
        CommandArg::new("voucher.payment.amount", "", NamedCLType::U512, true, false),
        CommandArg::new("voucher.names.label", "", NamedCLType::String, true, true),
        CommandArg::new("voucher.names.owner", "", NamedCLType::Key, true, true),
        CommandArg::new(
            "voucher.names.token_expiration",
            "",
            NamedCLType::U64,
            true,
            true
        ),
        CommandArg::new(
            "voucher.voucher_expiration",
            "",
            NamedCLType::U64,
            true,
            false
        ),
        CommandArg::new("signature", "", NamedCLType::U8, true, true),
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
