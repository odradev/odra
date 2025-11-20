//! This module defines various types of entry points for smart contracts
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use casper_types::{CLType, CLTyped, Parameter};

/// Represents different kinds of entry points in a contract.
pub enum EntryPoint {
    /// A regular entry point.
    Regular {
        /// The name of the entry point.
        name: &'static str,
        /// The arguments for the entry point.
        args: Vec<Option<Parameter>>,
        /// The return type of the entry point.
        ret_ty: CLType,
        /// Indicates if the entry point is non-reentrant.
        is_non_reentrant: bool,
        /// Indicates if the entry point is payable.
        is_payable: bool
    },
    /// A constructor entry point.
    Constructor {
        /// The arguments for the constructor.
        args: Vec<Option<Parameter>>
    },
    /// A template entry point, used in a factory pattern.
    Template {
        /// The name of the entry point.
        name: &'static str,
        /// The arguments for the entry point.
        args: Vec<Option<Parameter>>,
        /// The return type of the entry point.
        ret_ty: CLType
    },
    /// A factory entry point.
    Factory {
        /// The arguments for the factory.
        args: Vec<Option<Parameter>>
    },
    /// A factory upgrade entry point.
    FactoryUpgrade {
        /// The arguments for the factory.
        args: Vec<Option<Parameter>>
    },
    /// A factory batch upgrade entry point.
    FactoryBatchUpgrade,
    /// An upgrader entry point.
    Upgrader {
        /// The arguments for the upgrader entry point.
        args: Vec<Option<Parameter>>
    }
}

/// Extension trait for adding entry points to `casper_types::EntryPoints`.
pub trait EntityEntryPointsExt {
    /// Adds an entry point to the collection.
    fn add<T: Into<casper_types::EntityEntryPoint>>(&mut self, ep: T);
}

impl EntityEntryPointsExt for casper_types::EntryPoints {
    fn add<T: Into<casper_types::EntityEntryPoint>>(&mut self, ep: T) {
        self.add_entry_point(ep.into());
    }
}

impl From<EntryPoint> for casper_types::EntityEntryPoint {
    fn from(val: EntryPoint) -> Self {
        match val {
            EntryPoint::Regular {
                name, args, ret_ty, ..
            } => entry_point(name, args, ret_ty),
            EntryPoint::Constructor { args } => constructor(args),
            EntryPoint::Template { name, args, ret_ty } => template(name, args, ret_ty),
            EntryPoint::Factory { args } => factory(args),
            EntryPoint::Upgrader { args } => upgrader(args),
            EntryPoint::FactoryUpgrade { args } => factory_upgrade(args),
            EntryPoint::FactoryBatchUpgrade => factory_batch_upgrade()
        }
    }
}

impl TryFrom<EntryPoint> for crate::contract_def::Entrypoint {
    type Error = &'static str;

    fn try_from(val: EntryPoint) -> Result<Self, Self::Error> {
        match val {
            EntryPoint::Regular { name, args, ret_ty, is_non_reentrant, is_payable } => Ok(crate::contract_def::Entrypoint {
                name: crate::prelude::string::String::from(name),
                args: convert_args(args),
                is_mutable: true,
                return_ty: ret_ty,
                ty: crate::contract_def::EntrypointType::Public,
                attributes: match (is_non_reentrant, is_payable) {
                    (true, true) => vec![
                        crate::contract_def::EntrypointAttribute::NonReentrant,
                        crate::contract_def::EntrypointAttribute::Payable
                    ],
                    (true, false) => vec![
                        crate::contract_def::EntrypointAttribute::NonReentrant
                    ],
                    (false, true) => vec![
                        crate::contract_def::EntrypointAttribute::Payable
                    ],
                    (false, false) => vec![]
                }
            }),
            EntryPoint::Constructor { args } => Ok(crate::contract_def::Entrypoint {
                name: crate::prelude::String::from("init"),
                args: convert_args(args),
                is_mutable: true,
                return_ty: <() as crate::casper_types::CLTyped>::cl_type(),
                ty: crate::contract_def::EntrypointType::Constructor,
                attributes: crate::prelude::vec![]
            }),
            EntryPoint::Factory { args } => Ok(crate::contract_def::Entrypoint {
                name: crate::prelude::string::String::from("new_contract"),
                args: convert_args(args),
                is_mutable: true,
                return_ty: <(crate::prelude::Address, crate::casper_types::URef) as crate::casper_types::CLTyped>::cl_type(),
                ty: crate::contract_def::EntrypointType::Public,
                attributes: crate::prelude::vec![]
            }),
            EntryPoint::FactoryUpgrade { args } => Ok(crate::contract_def::Entrypoint {
                name: crate::prelude::String::from("upgrade_child_contract"),
                args: convert_args(args),
                is_mutable: true,
                return_ty: <() as crate::casper_types::CLTyped>::cl_type(),
                ty: crate::contract_def::EntrypointType::Public,
                attributes: crate::prelude::vec![]
            }),
            EntryPoint::FactoryBatchUpgrade => Ok(crate::contract_def::Entrypoint {
                name: crate::prelude::String::from("batch_upgrade_child_contract"),
                args: crate::prelude::vec![
                    crate::contract_def::Argument {
                        name: crate::prelude::String::from("args"),
                        ty: Vec::<u8>::cl_type(),
                        is_ref: false,
                        is_slice: false,
                        is_required: true
                    }
                ],
                is_mutable: true,
                return_ty: <() as crate::casper_types::CLTyped>::cl_type(),
                ty: crate::contract_def::EntrypointType::Public,
                attributes: crate::prelude::vec![]
            }),
            _ => Err("Conversion not implemented for this entry point type"),
        }
    }
}

fn convert_args(args: Vec<Option<Parameter>>) -> Vec<crate::contract_def::Argument> {
    args.into_iter()
        .flatten()
        .map(|p| crate::contract_def::Argument {
            name: crate::prelude::string::String::from(p.name()),
            ty: p.cl_type().clone(),
            is_ref: false,
            is_slice: false,
            is_required: true
        })
        .collect()
}

fn entry_point(
    name: &str,
    args: Vec<Option<Parameter>>,
    ret_ty: CLType
) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        name,
        args.into_iter().flatten().collect(),
        ret_ty,
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn constructor(args: Vec<Option<Parameter>>) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "init",
        args.into_iter().flatten().collect(),
        CLType::Unit,
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("constructor_group")]),
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn template(
    name: &str,
    args: Vec<Option<Parameter>>,
    ret_ty: CLType
) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        name,
        args.into_iter().flatten().collect(),
        ret_ty,
        casper_types::EntryPointAccess::Template,
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn factory(args: Vec<Option<Parameter>>) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "new_contract",
        args.into_iter().flatten().collect(),
        CLType::Tuple2([Box::new(CLType::Key), Box::new(CLType::URef)]),
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Factory,
        casper_types::EntryPointPayment::Caller
    )
}

fn factory_upgrade(args: Vec<Option<Parameter>>) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "upgrade_child_contract",
        args.into_iter().flatten().collect(),
        CLType::Unit,
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("factory_group")]),
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn factory_batch_upgrade() -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "batch_upgrade_child_contract",
        vec![casper_types::Parameter::new("args", Vec::<u8>::cl_type())],
        CLType::Unit,
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("factory_group")]),
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn upgrader(args: Vec<Option<Parameter>>) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "upgrade",
        args.into_iter().flatten().collect(),
        CLType::Unit,
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("upgrader_group")]),
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}
