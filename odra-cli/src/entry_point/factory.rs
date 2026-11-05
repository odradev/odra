//! The entry points of a factory (`#[odra::module(factory = on)]`) and the `odra_cfg_*` arguments
//! the generated ref of a factory adds to their calls, which the schema does not list.

use odra::casper_types::{CLValueError, RuntimeArgs};
use odra::consts;
use odra::schema::casper_contract_schema::Entrypoint;

pub(crate) const NEW_CONTRACT: &str = "new_contract";
pub(crate) const UPGRADE_CHILD_CONTRACT: &str = "upgrade_child_contract";
pub(crate) const BATCH_UPGRADE_CHILD_CONTRACT: &str = "batch_upgrade_child_contract";
pub(crate) const CONTRACT_NAME_ARG: &str = "contract_name";

/// Whether the entry points are the entry points of a factory.
pub(crate) fn is_factory(entry_points: &[Entrypoint]) -> bool {
    entry_points
        .iter()
        .any(|entry_point| entry_point.name == BATCH_UPGRADE_CHILD_CONTRACT)
}

/// Adds the `odra_cfg_*` arguments of a call of the factory entry point `entry_point` to `args`,
/// like the generated ref of the factory does. The arguments of other entry points are left as
/// they are.
pub(crate) fn insert_cfg_args(
    entry_point: &str,
    args: &mut RuntimeArgs
) -> Result<(), CLValueError> {
    match entry_point {
        NEW_CONTRACT => {
            // The child's package hash is stored under its name.
            let contract_name = args
                .get(CONTRACT_NAME_ARG)
                .map(|name| name.clone().into_t::<String>())
                .transpose()?
                .unwrap_or_default();
            args.insert(consts::IS_UPGRADABLE_ARG, true)?;
            args.insert(consts::IS_UPGRADE_ARG, false)?;
            args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true)?;
            args.insert(consts::PACKAGE_HASH_KEY_NAME_ARG, contract_name)?;
        }
        UPGRADE_CHILD_CONTRACT | BATCH_UPGRADE_CHILD_CONTRACT => {
            args.insert(consts::IS_FACTORY_UPGRADE_ARG, true)?;
            args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true)?;
            args.insert(consts::CREATE_UPGRADE_GROUP, false)?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use odra::casper_types::{runtime_args, CLValue};

    use super::*;

    fn arg<T: odra::casper_types::CLTyped + odra::casper_types::bytesrepr::ToBytes>(
        value: T
    ) -> Option<CLValue> {
        Some(CLValue::from_t(value).unwrap())
    }

    // The values are the ones the generated factory ref passes (see `odra-macros`, `ref_item.rs`).
    #[test]
    fn new_contract_stores_the_child_under_its_name() {
        let mut args = runtime_args! { "contract_name" => "FromTen".to_string(), "value" => 10u32 };
        insert_cfg_args(NEW_CONTRACT, &mut args).unwrap();
        assert_eq!(args.get("odra_cfg_is_upgradable").cloned(), arg(true));
        assert_eq!(args.get("odra_cfg_is_upgrade").cloned(), arg(false));
        assert_eq!(args.get("odra_cfg_allow_key_override").cloned(), arg(true));
        assert_eq!(
            args.get("odra_cfg_package_hash_key_name").cloned(),
            arg("FromTen".to_string())
        );
        assert_eq!(args.get("value").cloned(), arg(10u32));
    }

    #[test]
    fn child_upgrades_are_factory_upgrades() {
        for entry_point in [UPGRADE_CHILD_CONTRACT, BATCH_UPGRADE_CHILD_CONTRACT] {
            let mut args = RuntimeArgs::new();
            insert_cfg_args(entry_point, &mut args).unwrap();
            assert_eq!(args.get("odra_cfg_is_factory_upgrade").cloned(), arg(true));
            assert_eq!(args.get("odra_cfg_allow_key_override").cloned(), arg(true));
            assert_eq!(
                args.get("odra_cfg_create_upgrade_group").cloned(),
                arg(false)
            );
            assert_eq!(args.len(), 3);
        }
    }

    #[test]
    fn other_entry_points_are_left_as_they_are() {
        let mut args = runtime_args! { "value" => 10u32 };
        insert_cfg_args("increment", &mut args).unwrap();
        assert_eq!(args.len(), 1);
    }
}
