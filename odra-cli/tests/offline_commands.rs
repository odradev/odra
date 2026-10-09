//! Commands that don't talk to the network must work without any livenet configuration.
//!
//! The test binary doubles as the CLI under test (`harness = false`): with [`CHILD_MARKER`] set it
//! runs a small [`OdraCli`] on its own arguments, otherwise it spawns itself with various arguments,
//! with every `ODRA_*` variable cleared, stdin detached (no TTY to prompt on) and a scratch working
//! directory (so no `.env` file is picked up), and checks the outcome.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use odra::host::HostEnv;
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    DeployedContractsContainer, OdraCli
};

const CHILD_MARKER: &str = "__ODRA_CLI_OFFLINE_TEST_CHILD";
const MISSING_NODE_ADDRESS: &str =
    "Livenet env misconfigured! ODRA_CASPER_LIVENET_NODE_ADDRESS env var is missing.";
const NO_TTY: &str = "No interactive terminal to prompt on.";

struct NoopScenario;

impl ScenarioMetadata for NoopScenario {
    const NAME: &'static str = "noop";
    const DESCRIPTION: &'static str = "Does nothing";
}

impl Scenario for NoopScenario {
    fn run(&self, _: &HostEnv, _: &DeployedContractsContainer, _: Args) -> Result<(), Error> {
        Ok(())
    }
}

fn main() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        OdraCli::new()
            .about("Offline test CLI")
            .scenario(NoopScenario)
            .build()
            .run();
        return;
    }

    let tests: &[(&str, fn())] = &[
        ("help", help),
        ("subcommand_help", subcommand_help),
        ("usage_error", usage_error),
        ("completions", completions),
        ("config", config),
        ("config_json", config_json),
        (
            "network_command_still_requires_config",
            network_command_still_requires_config
        )
    ];
    for (name, test) in tests {
        print!("test {name} ... ");
        test();
        println!("ok");
    }
    let _ = std::fs::remove_dir_all(scratch_dir());
    println!("\ntest result: ok. {} passed", tests.len());
}

/// Runs the CLI under test with `args`, no `ODRA_*` variables, no TTY and a scratch working dir.
fn cli(args: &[&str]) -> Output {
    let dir = scratch_dir();
    let mut cmd = Command::new(std::env::current_exe().expect("test binary path"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("ODRA_") {
            cmd.env_remove(key);
        }
    }
    cmd.env(CHILD_MARKER, "1")
        .args(args)
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .expect("failed to spawn the CLI")
}

fn scratch_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("odra-cli-offline-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("failed to create a scratch dir");
    dir
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_no_livenet_error(output: &Output) {
    let all = format!("{}{}", stdout(output), stderr(output));
    assert!(
        !all.contains("misconfigured"),
        "unexpected livenet error:\n{all}"
    );
    assert!(!all.contains(NO_TTY), "unexpected prompt attempt:\n{all}");
}

fn help() {
    for flag in ["--help", "-h"] {
        let output = cli(&[flag]);
        assert!(output.status.success(), "{flag}: {output:?}");
        assert!(stdout(&output).contains("Offline test CLI"));
        assert!(stdout(&output).contains("completions"));
        assert_no_livenet_error(&output);
    }
    // No arguments at all prints the help (as a usage error).
    let output = cli(&[]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(stderr(&output).contains("Usage:"));
    assert_no_livenet_error(&output);
}

fn subcommand_help() {
    for args in [
        &["scenario", "--help"][..],
        &["scenario", "noop", "--help"],
        &["whoami", "--help"],
        &["deploy", "--help"],
        &["config", "--help"]
    ] {
        let output = cli(args);
        if args[0] == "deploy" {
            // No deploy script registered: a clap usage error, still without a livenet env.
            assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        } else {
            assert!(output.status.success(), "{args:?}: {output:?}");
            assert!(stdout(&output).contains("Usage:"), "{args:?}");
        }
        assert_no_livenet_error(&output);
    }
}

fn usage_error() {
    let output = cli(&["no-such-command"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(stderr(&output).contains("unrecognized subcommand"));
    assert_no_livenet_error(&output);
}

fn completions() {
    let output = cli(&["completions", "bash"]);
    assert!(output.status.success(), "{output:?}");
    assert!(stdout(&output).contains("complete -F"));
    assert_no_livenet_error(&output);
}

fn config() {
    let output = cli(&["config"]);
    assert!(output.status.success(), "{output:?}");
    let out = stdout(&output);
    assert!(out.contains("Livenet configuration:"), "{out}");
    assert!(
        out.contains("<not set> ($ODRA_CASPER_LIVENET_NODE_ADDRESS)"),
        "{out}"
    );
    assert!(
        out.contains("<not set> ($ODRA_CASPER_LIVENET_SECRET_KEY_PATH)"),
        "{out}"
    );
    assert!(out.contains("Caller address:  <unavailable>"), "{out}");
    assert!(out.contains(MISSING_NODE_ADDRESS), "{out}");
    assert!(!out.contains(NO_TTY), "{out}");
}

fn config_json() {
    let output = cli(&["--json", "config"]);
    assert!(output.status.success(), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["node_address"], serde_json::Value::Null);
    assert_eq!(json["caller_address"], serde_json::Value::Null);
    assert_eq!(json["problem"], MISSING_NODE_ADDRESS);
}

fn network_command_still_requires_config() {
    for args in [&["whoami"][..], &["scenario", "noop"]] {
        let output = cli(args);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {output:?}");
        let out = stdout(&output);
        assert!(out.contains(MISSING_NODE_ADDRESS), "{args:?}: {out}");
        assert!(out.contains(NO_TTY), "{args:?}: {out}");
    }
}
