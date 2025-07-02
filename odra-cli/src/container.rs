use std::{fs::File, io::Write, path::PathBuf, str::FromStr};

use chrono::{SecondsFormat, Utc};
use odra::{
    contract_def::HasIdent,
    host::{HostEnv, HostRef, HostRefLoader},
    prelude::{Address, Addressable},
    OdraContract
};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

pub const DEPLOYED_CONTRACTS_FILE: &str = "resources/contracts.toml";

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

/// Represents storage for deployed contracts.
/// This trait defines the methods for reading and writing contract data.
pub(crate) trait ContractStorage {
    /// Reads the contract data from the storage.
    fn read(&self) -> Result<ContractsData, ContractError>;
    /// Writes the contract data to the storage.
    fn write(&mut self, data: &ContractsData) -> Result<(), ContractError>;
}

/// Represents the data structure for storing deployed contracts in a TOML file.
pub(crate) struct FileContractStorage {
    file_path: PathBuf
}

impl FileContractStorage {
    pub fn new(custom_path: Option<PathBuf>) -> Result<Self, ContractError> {
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

        Ok(Self { file_path: path })
    }
}

impl ContractStorage for FileContractStorage {
    fn read(&self) -> Result<ContractsData, ContractError> {
        let file = std::fs::read_to_string(&self.file_path).map_err(ContractError::Io)?;
        toml::from_str(&file).map_err(ContractError::TomlDeserialize)
    }

    fn write(&mut self, data: &ContractsData) -> Result<(), ContractError> {
        let content = toml::to_string_pretty(&data).map_err(ContractError::TomlSerialize)?;
        let mut file = File::create(&self.file_path).map_err(ContractError::Io)?;
        file.write_all(content.as_bytes())
            .map_err(ContractError::Io)?;
        Ok(())
    }
}

/// This trait defines the methods for providing access to deployed contracts.
pub trait ContractProvider {
    /// Gets a reference to the contract.
    ///
    /// Returns a reference to the contract if it is found, otherwise returns an error.
    fn contract_ref<T: OdraContract + 'static>(
        &self,
        env: &HostEnv
    ) -> Result<T::HostRef, ContractError>;

    /// Returns a list of all deployed contracts with their names and addresses.
    fn all_contracts(&self) -> Vec<(String, Address)>;

    /// Returns the contract address.
    fn address_by_name(&self, name: &str) -> Option<Address>;
}

/// Struct representing the deployed contracts.
///
/// This struct is used to store the contracts name and address at the deploy
/// time and to retrieve a reference to the contract at runtime.
///
/// The data is stored in a TOML file `deployed_contracts.toml` in the
/// `{project_root}/resources` directory.
pub struct DeployedContractsContainer {
    data: ContractsData,
    storage: Box<dyn ContractStorage>
}

impl DeployedContractsContainer {
    /// Creates a new instance.
    pub(crate) fn instance(storage: impl ContractStorage + 'static) -> Self {
        match storage.read() {
            Ok(data) => Self {
                data,
                storage: Box::new(storage)
            },
            Err(_) => Self {
                data: Default::default(),
                storage: Box::new(storage)
            }
        }
    }

    /// Adds a contract to the container.
    pub fn add_contract<T: HostRef + HasIdent>(
        &mut self,
        contract: &T
    ) -> Result<(), ContractError> {
        self.data.add_contract::<T>(contract.address());
        self.storage.write(&self.data)
    }
}

impl ContractProvider for DeployedContractsContainer {
    fn contract_ref<T: OdraContract + 'static>(
        &self,
        env: &HostEnv
    ) -> Result<T::HostRef, ContractError> {
        self.data
            .contracts()
            .iter()
            .find(|c| c.name == T::HostRef::ident())
            .map(|c| Address::from_str(&c.package_hash).ok())
            .and_then(|opt| opt.map(|addr| <T as HostRefLoader<T::HostRef>>::load(env, addr)))
            .ok_or(ContractError::NotFound(T::HostRef::ident()))
    }

    fn all_contracts(&self) -> Vec<(String, Address)> {
        self.data
            .contracts()
            .iter()
            .filter_map(|c| { 
                Address::from_str(&c.package_hash)
                    .ok()
                    .map(|addr| (c.name.clone(), addr))
            })
            .collect()
    }

    fn address_by_name(&self, name: &str) -> Option<Address> {
        self.data
            .contracts()
            .iter()
            .find(|c| c.name == name)
            .and_then(|c| Address::from_str(&c.package_hash).ok())
    }
}

/// This struct represents a contract in the `deployed_contracts.toml` file.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct DeployedContract {
    name: String,
    package_hash: String
}

impl DeployedContract {
    fn new<T: HasIdent>(address: Address) -> Self {
        Self {
            name: T::ident(),
            package_hash: address.to_string()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ContractsData {
    #[serde(alias = "time")]
    last_updated: String,
    contracts: Vec<DeployedContract>
}

impl Default for ContractsData {
    fn default() -> Self {
        Self {
            last_updated: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            contracts: Vec::new()
        }
    }
}

impl ContractsData {
    pub fn add_contract<T: HasIdent>(&mut self, address: Address) {
        let contract = DeployedContract::new::<T>(address);
        self.contracts.retain(|c| c.name != contract.name);
        self.contracts.push(contract);
        self.last_updated = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    }

    fn contracts(&self) -> &Vec<DeployedContract> {
        &self.contracts
    }
}
