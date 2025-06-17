use odra::{
    host::{Deployer, HostEnv},
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
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) = container.contract_ref::<Self::Contract>(env) {
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy(env, args)?;
            container.add_contract(&contract)?;
            Ok(contract)
        }
    }
}

impl<T: OdraContract + Deployer<T> + 'static> DeployerExt for T {
    type Contract = T;
}
