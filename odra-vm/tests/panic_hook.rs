//! Checks what the OdraVM panic hook prints. A test cannot read its own panic output, so each
//! scenario runs in a child process (this test binary filtered to the `child` test) and the
//! parent asserts the child's stderr.

use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::RuntimeArgs;
use odra_core::entry_point_callback::{EntryPoint, EntryPointsCaller};
use odra_core::host::HostEnv;
use odra_core::prelude::*;
use odra_core::{CallDef, ContractEnv};
use odra_vm::{OdraVm, OdraVmHost};
use std::process::Command;

const SCENARIO_ENV: &str = "ODRA_PANIC_HOOK_SCENARIO";

#[test]
fn unwrap_after_a_contract_call_prints_the_standard_message() {
    let stderr = run_child("unwrap");
    assert!(
        stderr.contains("called `Option::unwrap()` on a `None` value"),
        "{stderr}"
    );
    assert!(stderr.contains("panicked at"), "{stderr}");
}

#[test]
fn failed_assertion_prints_the_standard_message() {
    let stderr = run_child("assert");
    assert!(
        stderr.contains("assertion `left == right` failed"),
        "{stderr}"
    );
    assert!(stderr.contains("panicked at"), "{stderr}");
}

#[test]
fn revert_prints_the_error_name() {
    let stderr = run_child("revert");
    assert!(stderr.contains("💣 Boom(7)"), "{stderr}");
    assert!(!stderr.contains("panicked at"), "{stderr}");
}

#[test]
fn revert_in_a_nested_call_prints_the_error_name() {
    let stderr = run_child("nested_revert");
    assert!(stderr.contains("💣 Boom(7)"), "{stderr}");
    assert!(!stderr.contains("panicked at"), "{stderr}");
}

#[test]
fn hook_installed_before_the_first_call_still_runs() {
    let stderr = run_child("user_hook");
    assert!(stderr.contains("user hook: boom"), "{stderr}");
}

/// Runs a scenario when started by [run_child], does nothing in a regular test run.
#[test]
fn child() {
    let Ok(scenario) = std::env::var(SCENARIO_ENV) else {
        return;
    };
    match scenario.as_str() {
        "unwrap" => {
            call(&test_env(), "ok").unwrap();
            std::hint::black_box(Option::<u8>::None).unwrap();
        }
        "assert" => {
            call(&test_env(), "ok").unwrap();
            assert_eq!(1, 2);
        }
        "revert" => {
            assert!(call(&test_env(), "fail").is_err());
        }
        "nested_revert" => {
            assert!(call(&test_env(), "nested").is_err());
        }
        "user_hook" => {
            std::panic::set_hook(Box::new(|info| {
                let msg = info.payload().downcast_ref::<&str>().unwrap_or(&"");
                eprintln!("user hook: {msg}");
            }));
            call(&test_env(), "ok").unwrap();
            panic!("boom");
        }
        _ => panic!("unknown scenario {scenario}")
    }
}

fn run_child(scenario: &str) -> String {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child", "--nocapture", "--test-threads=1"])
        .env(SCENARIO_ENV, scenario)
        .env_remove("RUST_BACKTRACE")
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn test_env() -> (HostEnv, Address) {
    let env = HostEnv::new(OdraVmHost::new(OdraVm::new()));
    let entry_points = ["ok", "fail", "nested"]
        .into_iter()
        .map(|name| EntryPoint::new(String::from(name), vec![]))
        .collect();
    let caller = EntryPointsCaller::new(env.clone(), entry_points, entry_point);
    let address = env
        .new_contract("Contract", RuntimeArgs::new(), caller)
        .unwrap();
    (env, address)
}

fn call((env, address): &(HostEnv, Address), entry_point: &str) -> OdraResult<()> {
    env.call_contract(
        *address,
        CallDef::new(entry_point, true, RuntimeArgs::new())
    )
}

fn entry_point(env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes> {
    match call_def.entry_point() {
        "fail" => env.revert(OdraError::user(7, "Boom")),
        "nested" => env.call_contract::<()>(
            env.self_address(),
            CallDef::new("fail", true, RuntimeArgs::new())
        ),
        _ => {}
    }
    Ok(Bytes::from(().to_bytes().unwrap()))
}
