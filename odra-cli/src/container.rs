use std::{fs::File, io::Write, path::PathBuf, str::FromStr};

use chrono::{DateTime, SecondsFormat, Utc};
use odra::{
    contract_def::HasIdent,
    host::{HostEnv, HostRef, HostRefLoader},
    Address, OdraContract
};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

const DEPLOYED_CONTRACTS_FILE: &str = "resources/deployed_contracts.toml";

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("TOML serialization error")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("TOML deserialization error")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("Couldn't read file")]
    Io(#[from] std::io::Error),
    #[error("Couldn't find contract `{0}`")]
    NotFound(String)
}

/// Struct representing the deployed contracts.
///
/// This struct is used to store the contracts name and address at the deploy
/// time and to retrieve a reference to the contract at runtime.
///
/// The data is stored in a TOML file `deployed_contracts.toml` in the
/// `{project_root}/resources` directory.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeployedContractsContainer {
    time: String,
    contracts: Vec<DeployedContract>
}

impl DeployedContractsContainer {
    /// Creates a new instance.
    pub(crate) fn new() -> Result<Self, ContractError> {
        Self::handle_previous_version()?;
        let now: DateTime<Utc> = Utc::now();
        Ok(Self {
            time: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            contracts: Vec::new()
        })
    }

    /// Adds a contract to the container.
    pub fn add_contract<T: HostRef + HasIdent>(
        &mut self,
        contract: &T
    ) -> Result<(), ContractError> {
        self.contracts
            .push(DeployedContract::new::<T>(contract.address()));
        self.update()
    }

    /// Gets reference to the contract.
    ///
    /// Returns a reference to the contract if it is found in the list, otherwise returns an error.
    pub fn get_ref<T: OdraContract + 'static>(
        &self,
        env: &HostEnv
    ) -> Result<T::HostRef, ContractError> {
        self.contracts
            .iter()
            .find(|c| c.name == T::HostRef::ident())
            .map(|c| Address::from_str(&c.package_hash).ok())
            .and_then(|opt| opt.map(|addr| <T as HostRefLoader<T::HostRef>>::load(env, addr)))
            .ok_or(ContractError::NotFound(T::HostRef::ident()))
    }

    /// Returns the contract address.
    pub fn address(&self, name: &str) -> Option<Address> {
        self.contracts
            .iter()
            .find(|c| c.name == name)
            .and_then(|c| Address::from_str(&c.package_hash).ok())
    }

    /// Load from the file.
    pub(crate) fn load() -> Result<Self, ContractError> {
        let path = Self::file_path()?;
        let file = std::fs::read_to_string(path).map_err(ContractError::Io)?;

        let result = toml::from_str(&file).map_err(ContractError::TomlDeserialize)?;
        Ok(result)
    }

    /// Backup previous version of the file.
    pub(crate) fn handle_previous_version() -> Result<(), ContractError> {
        if let Ok(deployed_contracts) = Self::load() {
            // Build new file name.
            let date = deployed_contracts.time();
            let mut path = project_root::get_project_root().map_err(ContractError::Io)?;
            path.push(format!("{}.{}", DEPLOYED_CONTRACTS_FILE, date));

            // Store previous version under new file name.
            deployed_contracts.save_at(&path)?;

            // Remove old file.
            std::fs::remove_file(path).map_err(ContractError::Io)?;
        }
        Ok(())
    }

    /// Save the file at the given path.
    fn save_at(&self, file_path: &PathBuf) -> Result<(), ContractError> {
        let content = toml::to_string_pretty(&self).map_err(ContractError::TomlSerialize)?;
        let mut file = File::create(file_path).map_err(ContractError::Io)?;

        file.write_all(content.as_bytes())
            .map_err(ContractError::Io)?;
        Ok(())
    }

    /// Return creation time.
    fn time(&self) -> &str {
        &self.time
    }

    /// Update the file.
    fn update(&self) -> Result<(), ContractError> {
        let path = Self::file_path()?;
        self.save_at(&path)
    }

    fn file_path() -> Result<PathBuf, ContractError> {
        let mut path = project_root::get_project_root().map_err(ContractError::Io)?;
        path.push(DEPLOYED_CONTRACTS_FILE);

        Ok(path)
    }
}

/// This struct represents a contract in the `deployed_contracts.toml` file.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct DeployedContract {
    name: String,
    package_hash: String
}

impl DeployedContract {
    fn new<T: HasIdent>(address: &Address) -> Self {
        Self {
            name: T::ident(),
            package_hash: address.to_string()
        }
    }
}
