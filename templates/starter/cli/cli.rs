//! CLI tool for deploying and interacting with smart contracts.

use odra::host::HostEnv;
use odra_cli::{
    cspr,
    DeployerExt,
    deploy::DeployScript,
    DeployedContractsContainer, OdraCli,
};

/// Deploys all project contracts.
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contracts")
        .deploy(ContractsDeployScript)
        .build()
        .run();
}
