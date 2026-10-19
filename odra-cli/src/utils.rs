use odra::{
    contract_def::HasIdent,
    host::{Deployer, HostEnv, InstallConfig},
    prelude::Addressable,
    OdraContract
};

use crate::{
    container::{ContractError, ContractStorage, FileContractStorage},
    ContractProvider, DeployedContractsContainer
};
use std::path::PathBuf;

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

/// Loads already deployed contracts from a contracts file written by the `deploy` command.
///
/// Saves copying package hashes out of `resources/contracts.toml` into a script:
///
/// ```ignore
/// let token = MyToken::load_from_default_file(&env)?;
/// let token = MyToken::load_from_file(&env, "resources/casper-test-contracts.toml")?;
/// ```
pub trait ContractLoaderExt: Sized {
    /// Contract that implements OdraContract for Self.
    type Contract: OdraContract + 'static;

    /// Loads the contract from the contracts file at `path`.
    ///
    /// A relative `path` is resolved against the project root, like the `--contracts-toml` option
    /// of the CLI.
    fn load_from_file(
        env: &HostEnv,
        path: impl Into<PathBuf>
    ) -> Result<<Self::Contract as OdraContract>::HostRef, crate::deploy::Error> {
        Self::load_from_file_named(env, path, None)
    }

    /// Loads the contract registered under `package_name` from the contracts file at `path`.
    fn load_from_file_named(
        env: &HostEnv,
        path: impl Into<PathBuf>,
        package_name: Option<String>
    ) -> Result<<Self::Contract as OdraContract>::HostRef, crate::deploy::Error> {
        let container = container_from_file(path.into())?;
        Ok(container.contract_ref_named::<Self::Contract>(env, package_name)?)
    }

    /// Loads the contract from the default contracts file: `resources/contracts.toml`, or
    /// `resources/<chain>-contracts.toml` when `ODRA_CASPER_LIVENET_CHAIN_NAME` is set.
    fn load_from_default_file(
        env: &HostEnv
    ) -> Result<<Self::Contract as OdraContract>::HostRef, crate::deploy::Error> {
        Self::load_from_file(env, get_default_contracts_file())
    }
}

impl<T: OdraContract + 'static> ContractLoaderExt for T {
    type Contract = T;
}

/// Opens an existing contracts file. A relative `path` is resolved against the project root.
///
/// Unlike the container used by the `deploy` command, this neither creates the file nor hides a
/// broken one: a missing or malformed file is an error, not an empty set of contracts.
pub(crate) fn container_from_file(
    path: PathBuf
) -> Result<DeployedContractsContainer, ContractError> {
    let storage = FileContractStorage::new(Some(path))?;
    let data = storage.read()?;
    Ok(DeployedContractsContainer::with_data(data, storage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::ContractProvider;

    fn temp_file(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("odra-cli-{}-{name}", std::process::id()));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn missing_file_is_an_error() {
        let path = std::env::temp_dir().join("odra-cli-does-not-exist.toml");
        let err = container_from_file(path).err().unwrap();
        assert!(matches!(err, ContractError::Io(_)), "{err:?}");
    }

    #[test]
    fn malformed_file_is_an_error() {
        let path = temp_file("bad.toml", "this is not = [toml");
        let err = container_from_file(path).err().unwrap();
        assert!(matches!(err, ContractError::TomlDeserialize(_)), "{err:?}");
    }

    #[test]
    fn contracts_are_read_from_the_file() {
        let path = temp_file(
            "ok.toml",
            r#"
time = "2026-09-17"

[[contracts]]
name = "Token"
package_name = "Token"
package_hash = "hash-0101010101010101010101010101010101010101010101010101010101010101"
"#
        );
        let container = container_from_file(path).unwrap();
        assert!(container.address_by_name("Token").is_some());
        assert!(container.address_by_name("Other").is_none());
    }
}
