//! The VM is freed with the last `HostEnv` using it: the contracts a backend keeps must not
//! hold the environment back.

use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::RuntimeArgs;
use odra_core::entry_point_callback::{EntryPoint, EntryPointsCaller};
use odra_core::host::HostEnv;
use odra_core::prelude::*;
use odra_core::{CallDef, ContractEnv};
use odra_vm::{OdraVm, OdraVmHost};
use std::rc::Weak;

#[test]
fn deployed_contract_does_not_keep_the_vm_alive() {
    assert_vm_dropped(|env| {
        let address = deploy(env, "Contract", contract());
        call(env, address, "ok");
    });
}

#[test]
fn factory_children_do_not_keep_the_vm_alive() {
    assert_vm_dropped(|env| {
        let factory = deploy(env, "Factory", factory());
        call(env, factory, "new_child");
        call(env, factory, "upgrade_child");
    });
}

#[test]
fn snapshot_does_not_keep_the_vm_alive() {
    assert_vm_dropped(|env| {
        deploy(env, "Contract", contract());
        env.take_snapshot();
        let factory = deploy(env, "Factory", factory());
        call(env, factory, "new_child");
        env.restore_snapshot();
        env.take_snapshot();
    });
}

#[test]
fn loaded_contract_does_not_keep_the_vm_alive() {
    assert_vm_dropped(|env| {
        let address = deploy(env, "Contract", contract());
        // What `HostRefLoader::load` does.
        env.register_contract(address, String::from("Contract"), contract());
        call(env, address, "ok");
    });
}

fn assert_vm_dropped(scenario: impl FnOnce(&HostEnv)) {
    let vm = OdraVm::new();
    let weak: Weak<OdraVm> = Rc::downgrade(&vm);
    let env = HostEnv::new(OdraVmHost::new(vm));
    scenario(&env);
    drop(env);
    assert!(weak.upgrade().is_none(), "the VM outlived its environment");
}

fn deploy(env: &HostEnv, name: &str, caller: EntryPointsCaller) -> Address {
    env.new_contract(name, RuntimeArgs::new(), caller).unwrap()
}

fn call(env: &HostEnv, address: Address, entry_point: &str) {
    env.call_contract::<()>(address, CallDef::new(entry_point, true, RuntimeArgs::new()))
        .unwrap();
}

fn contract() -> EntryPointsCaller {
    let entry_points = vec![EntryPoint::new(String::from("ok"), vec![])];
    EntryPointsCaller::new(entry_points, |_, _| unit())
}

/// A factory deploying and upgrading a child named `Child`.
fn factory() -> EntryPointsCaller {
    let entry_points = ["new_contract", "new_child", "upgrade_child"]
        .into_iter()
        .map(|name| EntryPoint::new(String::from(name), vec![]))
        .collect();
    EntryPointsCaller::new(entry_points, |env: ContractEnv, call_def| {
        match call_def.entry_point() {
            "new_child" => {
                env.new_child_contract("Child", RuntimeArgs::new(), contract)?;
            }
            "upgrade_child" => {
                env.upgrade_child_contract("Child", RuntimeArgs::new(), contract)?;
            }
            _ => {}
        }
        unit()
    })
}

fn unit() -> OdraResult<Bytes> {
    Ok(Bytes::from(().to_bytes().unwrap()))
}
