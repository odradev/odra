use odra::{
    contract_def::HasIdent,
    host::{Deployer, HostEnv, InstallConfig},
    prelude::Addressable,
    OdraContract
};

use crate::{ContractProvider, DeployedContractsContainer};

/// Logs a message to the console.
pub fn log<T: ToString>(msg: T) {
    prettycli::info(&msg.to_string());
}

/// Trait that extends the functionality of OdraContract to include deployment capabilities.
pub trait DeployerExt: Sized {
    /// Contract that implements OdraContract and Deployer for Self
    type Contract: OdraContract + 'static + Deployer<Self::Contract>;

    /// Load an existing contract instance from container or deploy a new one.
    fn load_or_deploy(
        env: &HostEnv,
        name: Option<String>,
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) = container.contract_ref::<Self::Contract>(env, name) {
            prettycli::info(&format!(
                "Using existing contract {} at address {:?}",
                <Self::Contract as OdraContract>::HostRef::ident(),
                contract.address()
            ));
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy(env, args)?;
            container.add_contract(&contract)?;
            Ok(contract)
        }
    }

    /// Load an existing contract instance from container or deploy a new one with a custom configuration.
    fn load_or_deploy_with_cfg(
        env: &HostEnv,
        name: Option<String>,
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        cfg: InstallConfig,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) = container.contract_ref::<Self::Contract>(env, name) {
            prettycli::info(&format!(
                "Using existing contract {} at address {:?}",
                <Self::Contract as OdraContract>::HostRef::ident(),
                contract.address()
            ));
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy_with_cfg(env, args, cfg)?;
            container.add_contract(&contract)?;
            Ok(contract)
        }
    }
}

impl<T: OdraContract + Deployer<T> + 'static> DeployerExt for T {
    type Contract = T;
}
