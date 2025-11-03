//! This module defines various types of entry points for smart contracts
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use casper_types::{CLType, Parameter};

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
    FactoryUpgrade {
        /// The arguments for the factory.
        args: Vec<Option<Parameter>>
    },
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
            EntryPoint::FactoryUpgrade { args } => factory_upgrade(args)
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
        "factory",
        args.into_iter().flatten().collect(),
        CLType::Tuple2([Box::new(CLType::Key), Box::new(CLType::URef)]),
        casper_types::EntryPointAccess::Public,
        casper_types::EntryPointType::Factory,
        casper_types::EntryPointPayment::Caller
    )
}

fn factory_upgrade(args: Vec<Option<Parameter>>) -> casper_types::EntityEntryPoint {
    casper_types::EntityEntryPoint::new(
        "factory_upgrade",
        args.into_iter().flatten().collect(),
        CLType::List(Box::new(CLType::Key)),
        casper_types::EntryPointAccess::Groups(vec![casper_types::Group::new("upgrader_group")]),
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
