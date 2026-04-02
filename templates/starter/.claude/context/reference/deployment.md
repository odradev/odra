# Deployment Reference

## Environment Variables

Set these before running the CLI binary against a live node. Copy `.env.sample` to `.env`
and fill in your values:

```env
ODRA_CASPER_LIVENET_SECRET_KEY_PATH=/path/to/secret_key.pem
ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101
ODRA_CASPER_LIVENET_EVENTS_URL=http://localhost:18101/events/main
ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1
```

| Variable | Description |
|---|---|
| `SECRET_KEY_PATH` | Path to PEM file for signing transactions |
| `NODE_ADDRESS` | RPC endpoint of the Casper node |
| `EVENTS_URL` | SSE events endpoint (for waiting on deploys) |
| `CHAIN_NAME` | `casper-net-1` (nctl), `casper-test` (testnet), `casper` (mainnet) |

Load at runtime with:

```bash
set -a && source .env && set +a && cargo run --bin cli --features=livenet -- deploy
```

## `load_or_deploy` Pattern

In your deploy script, use `ContractType::load_or_deploy(env, args, container, gas)`.
On first run it deploys and saves the address. On subsequent runs it loads the existing
contract from `resources/<chain>-contracts.toml`:

```rust
use odra_cli::DeployerExt;

fn deploy(&self, env: &HostEnv, container: &mut DeployedContractsContainer)
    -> Result<(), Error>
{
    let _token = MyToken::load_or_deploy(
        env,
        MyTokenInitArgs {
            name: "My Token".to_string(),
            symbol: "MTK".to_string(),
            decimals: 18,
            initial_supply: Some(1_000_000.into()),
        },
        container,
        cspr!(350),   // gas budget in motes; cspr!(350) = 350 CSPR
    )?;
    Ok(())
}
```

## `OdraCli` Builder

Wire up the CLI in `cli/cli.rs`:

```rust
OdraCli::new()
    .about("My project CLI")
    .deploy(MyDeployScript)          // deploy script
    .contract::<MyToken>()           // expose contract entry points as commands
    .scenario::<CheckBalance>(CheckBalance) // register a scenario
    .build()
    .run();
```

## Writing a Scenario

A scenario is a named set of actions against deployed contracts. Implement both
`Scenario` and `ScenarioMetadata` traits:

```rust
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
};
use odra::schema::casper_contract_schema::NamedCLType;

pub struct CheckBalanceScenario;

impl Scenario for CheckBalanceScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("account", "Account address to check", NamedCLType::Key)
                .required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        let token = container.contract_ref::<MyToken>(env)?;
        let account: Address = args.get_single::<Address>("account")?;
        env.set_gas(50_000_000);
        let balance = token.try_balance_of(&account)?;
        odra_cli::log(format!("Balance: {}", balance));
        Ok(())
    }
}

impl ScenarioMetadata for CheckBalanceScenario {
    const NAME: &'static str = "check-balance";
    const DESCRIPTION: &'static str = "Check the token balance of an account";
}
```

## Running the CLI

```bash
# Deploy (or load existing) contracts
cargo run --bin cli --features=livenet -- deploy

# Call a contract entry point interactively
cargo run --bin cli --features=livenet -- contract MyToken balance_of --owner <address>

# Run a scenario
cargo run --bin cli --features=livenet -- scenario check-balance --account <address>
```

## Network Differences

| Network | `CHAIN_NAME` | Node setup |
|---|---|---|
| nctl (local) | `casper-net-1` | Docker — use `/start-nctl` skill |
| Testnet | `casper-test` | Public node or self-hosted |
| Mainnet | `casper` | Public node or self-hosted |

Keys for nctl are in `nctl/assets/users/user-1/secret_key.pem` after NCTL starts.
