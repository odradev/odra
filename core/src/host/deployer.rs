//! Putting a contract on a backend from the host: [`Deployer`] with its [`InstallConfig`] and
//! [`UpgradeConfig`], the argument types of a constructor and an upgrader, and the
//! [`HostRefLoader`] impl that picks up an already deployed contract.

#[cfg(not(target_arch = "wasm32"))]
use crate::host::{EntryPointsCallerProvider, HostEnv, HostRef, HostRefLoader};
#[cfg(not(target_arch = "wasm32"))]
use crate::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::{consts, contract::OdraContract, contract_def::HasIdent};
use casper_types::RuntimeArgs;

/// A type which can deploy a contract.
///
/// Before any interaction with the contract, it must be deployed, either
/// on a virtual machine or on a real blockchain.
///
/// The `Deployer` trait provides a simple way to deploy a contract.
#[cfg(not(target_arch = "wasm32"))]
pub trait Deployer<R: OdraContract>: Sized {
    /// Deploys a contract with given init args.
    ///
    /// If the `init_args` is not [NoArgs], the contract is deployed and initialized
    /// by calling the constructor. Otherwise no constructor is called.
    ///
    /// The default [InstallConfig] is used for deployment.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy(env: &HostEnv, init_args: R::InitArgs) -> R::HostRef;

    /// Tries to deploy a contract with given init args.
    ///
    /// Similar to `deploy`, but returns a result instead of panicking.
    fn try_deploy(env: &HostEnv, init_args: R::InitArgs) -> OdraResult<R::HostRef>;

    /// Deploys a contract with given init args and configuration.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy_with_cfg(env: &HostEnv, init_args: R::InitArgs, cfg: InstallConfig) -> R::HostRef;

    /// Tries to deploy a contract with given init args and configuration.
    ///
    /// Similar to `deploy_with_cfg`, but returns a result instead of panicking.
    fn try_deploy_with_cfg(
        env: &HostEnv,
        init_args: R::InitArgs,
        cfg: InstallConfig
    ) -> OdraResult<R::HostRef>;

    /// Tries to upgrade a contract with given init args.
    fn try_upgrade(
        env: &HostEnv,
        address: Address,
        init_args: R::UpgradeArgs
    ) -> OdraResult<R::HostRef>;

    /// Tries to upgrade a contract with given init args and configuration
    fn try_upgrade_with_cfg(
        env: &HostEnv,
        address: Address,
        upgrade_args: R::UpgradeArgs,
        cfg: UpgradeConfig
    ) -> OdraResult<R::HostRef>;
}

/// A type which can be used as initialization arguments for a contract.
pub trait InitArgs: Into<RuntimeArgs> {}
/// A type which can be used as upgrade arguments for a contract.
pub trait UpgradeArgs: Into<RuntimeArgs> {}

/// Default implementation of [InitArgs]. Should be used when the contract
/// does not require initialization arguments.
///
/// Precisely, it means the constructor function has not been defined,
/// or does not require any arguments.
pub struct NoArgs;

impl InitArgs for NoArgs {}

impl UpgradeArgs for NoArgs {}

impl From<NoArgs> for RuntimeArgs {
    fn from(_: NoArgs) -> Self {
        RuntimeArgs::new()
    }
}

/// A configuration for a contract.
///
/// The configuration every contract written in Odra expects.
/// Read more: <https://odra.dev/docs/backends/casper/#wasm-arguments>
#[cfg(not(target_arch = "wasm32"))]
pub struct InstallConfig {
    /// Returns the package hash of the contract.
    ///
    /// Used to set the `odra_cfg_package_hash_key_name` key at the contract initialization.
    pub package_named_key: String,
    /// Returns true if the contract should be deployed as upgradable.
    ///
    /// If true, the `odra_cfg_is_upgradable` key is set to `true` at the contract initialization.
    pub is_upgradable: bool,
    /// If true and the key `odra_cfg_package_hash_key_name` already exists, it should be overwritten.
    pub allow_key_override: bool
}

/// A configuration for upgrading contract.
///
/// The configuration every contract upgrade written in Odra expects.
/// Read more: <https://odra.dev/docs/backends/casper/#wasm-arguments>
#[cfg(not(target_arch = "wasm32"))]
pub struct UpgradeConfig {
    /// Returns the package hash of the contract.
    ///
    /// Used to set the `odra_cfg_package_hash_key_name` key at the contract initialization.
    pub package_named_key: String,
    /// Create a new upgrade group for the contract. Set it to `true` if you want to upgrade a contract
    /// Which was deployed using Odra 2.2 or earlier, or not using Odra at all.
    pub force_create_upgrade_group: bool,
    /// If true and the key `odra_cfg_package_hash_key_name` already exists, it should be overwritten.
    pub allow_key_override: bool
}

#[cfg(not(target_arch = "wasm32"))]
impl InstallConfig {
    /// Returns new InstallConfig
    pub fn new<T: HasIdent>(is_upgradable: bool, allow_key_override: bool) -> Self {
        InstallConfig {
            package_named_key: T::contract_name(),
            is_upgradable,
            allow_key_override
        }
    }

    /// Returns new InstallConfig configured for default upgradable contract
    pub fn upgradable<T: HasIdent>() -> Self {
        InstallConfig::new::<T>(true, true)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl UpgradeConfig {
    /// Returns new UpgradeConfig with default values.
    /// It is by default upgradable and allows key override.
    pub fn new<T: HasIdent>() -> Self {
        UpgradeConfig {
            package_named_key: T::contract_name(),
            force_create_upgrade_group: false,
            allow_key_override: true
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<R: OdraContract> Deployer<R> for R {
    fn deploy(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs
    ) -> <R as OdraContract>::HostRef {
        let contract_ident = R::HostRef::ident();
        match Self::try_deploy(env, init_args) {
            Ok(contract) => contract,
            Err(OdraError::ExecutionError(ExecutionError::MissingArg)) => {
                core::panic!("Invalid init args for contract {}.", contract_ident)
            }
            Err(e) => core::panic!("Contract init failed {:?}", e)
        }
    }

    fn try_deploy(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        Self::try_deploy_with_cfg(
            env,
            init_args,
            InstallConfig::new::<<R as OdraContract>::HostRef>(false, true)
        )
    }

    fn deploy_with_cfg(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs,
        cfg: InstallConfig
    ) -> <R as OdraContract>::HostRef {
        let contract_ident = R::HostRef::ident();
        match Self::try_deploy_with_cfg(env, init_args, cfg) {
            Ok(contract) => contract,
            Err(OdraError::ExecutionError(ExecutionError::MissingArg)) => {
                core::panic!("Invalid init args for contract {}.", contract_ident)
            }
            Err(e) => core::panic!("Contract init failed {:?}", e)
        }
    }

    fn try_deploy_with_cfg(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs,
        cfg: InstallConfig
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        let contract_ident = R::HostRef::ident();
        // A contract that cannot emit one of its events is a programming error, like a missing
        // trait impl: it panics instead of returning an error, before anything is deployed.
        crate::contract_def::assert_events_can_be_emitted(&contract_ident, &R::contract_events());
        let caller = R::HostRef::entry_points_caller();

        let mut init_args = init_args.into();
        init_args.insert(consts::IS_UPGRADABLE_ARG, cfg.is_upgradable)?;
        init_args.insert(consts::IS_UPGRADE_ARG, false)?;
        init_args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, cfg.allow_key_override)?;
        init_args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            format!("{}_package_hash", cfg.package_named_key)
        )?;

        let address = env.new_contract(&contract_ident, init_args, caller)?;
        Ok(R::HostRef::new(address, env.clone()))
    }

    fn try_upgrade(
        env: &HostEnv,
        contract_to_upgrade: Address,
        upgrade_args: <R as OdraContract>::UpgradeArgs
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        Self::try_upgrade_with_cfg(
            env,
            contract_to_upgrade,
            upgrade_args,
            UpgradeConfig::new::<<R as OdraContract>::HostRef>()
        )
    }

    fn try_upgrade_with_cfg(
        env: &HostEnv,
        contract_to_upgrade: Address,
        upgrade_args: <R as OdraContract>::UpgradeArgs,
        cfg: UpgradeConfig
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        let mut upgrade_args = upgrade_args.into();
        upgrade_args.insert(consts::IS_UPGRADE_ARG, true)?;
        upgrade_args.insert(
            consts::PACKAGE_HASH_TO_UPGRADE_ARG,
            contract_to_upgrade.value()
        )?;
        upgrade_args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, cfg.allow_key_override)?;
        upgrade_args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            format!("{}_package_hash", cfg.package_named_key)
        )?;
        upgrade_args.insert(consts::CREATE_UPGRADE_GROUP, cfg.force_create_upgrade_group)?;
        let contract_ident = R::HostRef::ident();
        crate::contract_def::assert_events_can_be_emitted(&contract_ident, &R::contract_events());
        let entry_points_caller = R::HostRef::entry_points_caller();

        let address = env.upgrade_contract(
            &contract_ident,
            contract_to_upgrade,
            upgrade_args,
            entry_points_caller
        )?;
        Ok(HostRef::new(address, env.clone()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: OdraContract> HostRefLoader<T::HostRef> for T {
    fn load(env: &HostEnv, address: Address) -> T::HostRef {
        let caller = T::HostRef::entry_points_caller();
        let contract_name = T::HostRef::ident();
        env.register_contract(address, contract_name, caller);
        T::HostRef::new(address, env.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry_point_callback::EntryPointsCaller;
    use crate::host::MockHostContext;
    use crate::{ContractCallResult, ContractEnv, EventError};
    use casper_event_standard::EventInstance;
    use casper_types::account::AccountHash;
    use casper_types::bytesrepr::{Bytes, FromBytes};
    use casper_types::U512;
    use mockall::mock;
    use std::sync::Mutex;

    static IDENT_MTX: Mutex<()> = Mutex::new(());
    static EPC_MTX: Mutex<()> = Mutex::new(());

    mock! {
        TestRef {}
        impl HasIdent for TestRef {
            fn ident() -> String;
        }
        impl EntryPointsCallerProvider for TestRef {
            fn entry_points_caller() -> EntryPointsCaller;
        }
        impl HostRef for TestRef {
            fn new(address: Address, env: HostEnv) -> Self;
            fn with_tokens(&self, tokens: U512) -> Self;
            fn contract_address(&self) -> Address;
            fn env(&self) -> &HostEnv;
            fn get_event<T>(&self, index: i32) -> Result<T, EventError> where T: FromBytes + EventInstance + 'static;
            fn last_call(&self) -> ContractCallResult;
        }
    }

    impl crate::ContractRef for MockTestRef {
        fn new(_env: Rc<ContractEnv>, _address: Address) -> Self {
            unimplemented!()
        }
        fn address(&self) -> &Address {
            unimplemented!()
        }

        fn with_tokens(&self, _tokens: U512) -> Self {
            unimplemented!()
        }
    }

    impl crate::contract_def::HasEvents for MockTestRef {
        fn events() -> Vec<crate::contract_def::Event> {
            Vec::new()
        }
    }

    impl OdraContract for MockTestRef {
        type HostRef = MockTestRef;

        type ContractRef = MockTestRef;

        type InitArgs = NoArgs;

        type UpgradeArgs = NoArgs;
    }

    #[test]
    fn test_deploy_with_default_args() {
        // MockTestRef::ident() and  MockTestRef::entry_points_caller() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _i = IDENT_MTX.lock();
        let _e = EPC_MTX.lock();

        // stubs
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|| EntryPointsCaller::new(vec![], |_, _| Ok(Bytes::default())));

        // check if TestRef::new() is called exactly once
        let instance_ctx = MockTestRef::new_context();
        instance_ctx
            .expect()
            .times(1)
            .returning(|_, _| MockTestRef::default());

        let mut ctx = MockHostContext::new();
        ctx.expect_new_contract()
            .returning(|_, _, _| Ok(Address::Account(AccountHash::new([0; 32]))));
        ctx.expect_take_child_contracts().returning(Vec::new);
        // The event baseline of the new contract is read right after the deploy.
        ctx.expect_get_events_count().returning(|_| Ok(0));
        ctx.expect_get_native_events_count().returning(|_| Ok(0));
        let env = HostEnv::new(Rc::new(ctx));
        MockTestRef::deploy(&env, NoArgs);
    }

    #[test]
    fn test_load_ref() {
        // MockTestRef::ident() and MockTestRef::entry_points_caller() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _e = EPC_MTX.lock();
        let _i = IDENT_MTX.lock();

        // stubs
        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|| EntryPointsCaller::new(vec![], |_, _| Ok(Bytes::default())));
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let mut ctx = MockHostContext::new();
        ctx.expect_register_contract().returning(|_, _, _| ());
        ctx.expect_get_events_count().returning(|_| Ok(0));
        ctx.expect_get_native_events_count().returning(|_| Ok(0));

        // check if TestRef::new() is called exactly once
        let instance_ctx = MockTestRef::new_context();
        instance_ctx
            .expect()
            .times(1)
            .returning(|_, _| MockTestRef::default());

        let env = HostEnv::new(Rc::new(ctx));
        let address = Address::Account(AccountHash::new([0; 32]));
        MockTestRef::load(&env, address);
    }
}
