use std::{fs::File, io::Write, path::PathBuf, str::FromStr};

use chrono::{DateTime, SecondsFormat, Utc};
use odra::{
    contract_def::HasIdent,
    host::{HostEnv, HostRef, HostRefLoader},
    prelude::Address,
    OdraContract
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
    NotFound(String),
    #[error("Couldn't find schema file for contract `{0}`")]
    SchemaFileNotFound(String)
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
    contracts: Vec<DeployedContract>,
    #[serde(skip)]
    custom_path: Option<PathBuf>
}

impl DeployedContractsContainer {
    /// Creates a new instance.
    pub(crate) fn new(path: Option<PathBuf>) -> Result<Self, ContractError> {
        let now: DateTime<Utc> = Utc::now();
        let current_version = Self::load(path.clone());
        match current_version {
            Ok(mut container) => {
                // If the file already exists, we update the time.
                container.time = now.to_rfc3339_opts(SecondsFormat::Secs, true);
                container.update()?;
                container.custom_path = path;
                Ok(container)
            }
            Err(_) => Ok(Self {
                time: now.to_rfc3339_opts(SecondsFormat::Secs, true),
                contracts: Vec::new(),
                custom_path: path
            })
        }
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

    pub fn contracts(&self) -> Vec<(String, Address)> {
        self.contracts
            .iter()
            .map(|c| (c.name.clone(), Address::from_str(&c.package_hash).unwrap()))
            .collect()
    }

    /// Load from the file.
    pub fn load(path: Option<PathBuf>) -> Result<Self, ContractError> {
        let path = Self::file_path(path)?;
        let file = std::fs::read_to_string(path).map_err(ContractError::Io)?;

        let result = toml::from_str(&file).map_err(ContractError::TomlDeserialize)?;
        Ok(result)
    }

    /// Save the file at the given path.
    fn save_at(&self, file_path: &PathBuf) -> Result<(), ContractError> {
        let content = toml::to_string_pretty(&self).map_err(ContractError::TomlSerialize)?;
        let mut file = File::create(file_path).map_err(ContractError::Io)?;

        file.write_all(content.as_bytes())
            .map_err(ContractError::Io)?;
        Ok(())
    }

    /// Update the file.
    fn update(&self) -> Result<(), ContractError> {
        let custom_path = self.custom_path.clone();
        let path = Self::file_path(custom_path)?;
        self.save_at(&path)
    }

    fn file_path(custom_path: Option<PathBuf>) -> Result<PathBuf, ContractError> {
        let mut path = project_root::get_project_root().map_err(ContractError::Io)?;
        match &custom_path {
            Some(path_str) if !path_str.to_str().unwrap_or_default().is_empty() => {
                path.push(path_str);
            }
            _ => path.push(DEPLOYED_CONTRACTS_FILE)
        }
        if !path.exists() {
            let parent_path = path.parent().ok_or_else(|| {
                ContractError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent directory not found"
                ))
            })?;
            std::fs::create_dir_all(parent_path).map_err(ContractError::Io)?;
        }

        Ok(path)
    }
}

/// This struct represents a contract in the `deployed_contracts.toml` file.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeployedContract {
    pub name: String,
    pub package_hash: String
}

impl DeployedContract {
    fn new<T: HasIdent>(address: &Address) -> Self {
        Self {
            name: T::ident(),
            package_hash: address.to_string()
        }
    }
}
