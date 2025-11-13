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
        ret_ty: CLType
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
    FactoryUpgrade,
    /// A factory upgrade entry point.
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
            EntryPoint::Regular { name, args, ret_ty } => entry_point(name, args, ret_ty),
            EntryPoint::Constructor { args } => constructor(args),
            EntryPoint::Template { name, args, ret_ty } => template(name, args, ret_ty),
            EntryPoint::Factory { args } => factory(args),
            EntryPoint::Upgrader { args } => upgrader(args),
            EntryPoint::FactoryUpgrade => factory_upgrade(),
            EntryPoint::FactoryBatchUpgrade => factory_batch_upgrade()
        }
    }
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

fn factory_upgrade() -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "upgrade_child_contract",
        vec![casper_types::Parameter::new("args", Vec::<u8>::cl_type())],
        CLType::Unit,
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("factory_group")]),
        casper_types::EntryPointType::Called,
        casper_types::EntryPointPayment::Caller
    )
}

fn factory_batch_upgrade() -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "batch_upgrade_child_contract",
        vec![
            casper_types::Parameter::new("default_args", Vec::<u8>::cl_type()),
            casper_types::Parameter::new("names_to_upgrade", Vec::<u8>::cl_type()),
            casper_types::Parameter::new("specific_args", Vec::<u8>::cl_type()),
        ],
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
