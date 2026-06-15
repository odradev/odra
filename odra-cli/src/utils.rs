use odra::{
    contract_def::HasIdent,
    host::{Deployer, HostEnv, InstallConfig},
    prelude::Addressable,
    OdraContract
};

use crate::{ContractProvider, DeployedContractsContainer};

const DEFAULT_CONTRACTS_FILE: &str = "resources/contracts.toml";

/// Logs a message to the console.
pub fn log<T: ToString>(msg: T) {
    prettycli::info(&msg.to_string());
}

/// Returns the default contracts file path, checking for ODRA_CASPER_LIVENET_CHAIN_NAME
/// environment variable. If the variable exists, returns `resources/{network_name}-contracts.toml`,
/// otherwise returns the default contracts file path.
pub(crate) fn get_default_contracts_file() -> String {
    if let Ok(network_name) = std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME") {
        format!("resources/{}-contracts.toml", network_name)
    } else {
        DEFAULT_CONTRACTS_FILE.to_string()
    }
}

/// Trait that extends the functionality of OdraContract to include deployment capabilities.
pub trait DeployerExt: Sized {
    /// Contract that implements OdraContract and Deployer for Self
    type Contract: OdraContract + 'static + Deployer<Self::Contract>;

    /// Load an existing contract instance from container or deploy a new one.
    fn load_or_deploy(
        env: &HostEnv,
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) = container.contract_ref::<Self::Contract>(env) {
            prettycli::info(&format!(
                "Using existing contract {} at address {}",
                <Self::Contract as OdraContract>::HostRef::ident(),
                contract.address().to_string()
            ));
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy(env, args)?;
            // Set back to 0 to avoid unintedend sending huge amount of gas in consequtive calls 
            env.set_gas(0);
            container.add_contract(&contract)?;
            Ok(contract)
        }
    }

    /// Load an existing contract instance from container or deploy a new one with a custom configuration.
    fn load_or_deploy_with_cfg(
        env: &HostEnv,
        package_name: Option<String>,
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        cfg: InstallConfig,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) =
            container.contract_ref_named::<Self::Contract>(env, package_name.clone())
        {
            prettycli::info(&format!(
                "Using existing contract {} at address {}",
                <Self::Contract as OdraContract>::HostRef::ident(),
                contract.address().to_string()
            ));
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy_with_cfg(env, args, cfg)?;
            // Set back to 0 to avoid unintedend sending huge amount of gas in consequtive calls 
            env.set_gas(0);
            container.add_contract_named(&contract, package_name)?;
            Ok(contract)
        }
    }
}

impl<T: OdraContract + Deployer<T> + 'static> DeployerExt for T {
    type Contract = T;
}
