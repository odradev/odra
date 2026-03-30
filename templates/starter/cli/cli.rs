//! CLI tool for deploying and interacting with smart contracts.

use odra::host::HostEnv;
use odra_cli::{
    deploy::DeployScript,
    DeployedContractsContainer, OdraCli,
};

/// Deploys all project contracts.
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        _env: &HostEnv,
        _container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        // Add contract deployments here using:
        //   let _ = MyContract::load_or_deploy(&env, args, container, 250_000_000_000)?;
        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contracts")
        .deploy(ContractsDeployScript)
        // Register contracts with .contract::<MyContract>()
        // Register scenarios with .scenario(MyScenario)
        .build()
        .run();
}
