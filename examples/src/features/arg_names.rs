//! An argument may have any name, also the name of a variable of the code that Odra generates
//! around an entry point (`contract`, `env_rc`, `exec_env`, `named_args`, ...): the locals of the
//! generated code are prefixed, so they never clash with the arguments.
use odra::casper_types::U512;
use odra::prelude::*;
use odra::ContractRef;

/// A contract whose entry points take arguments named like the internals of the generated code.
#[odra::module]
pub struct ArgNames {
    value: Var<u32>,
    owner: Var<Address>
}

#[odra::module]
impl ArgNames {
    /// Sets the initial value and owner.
    pub fn init(&mut self, env: u32, contract: Address) {
        self.value.set(env);
        self.owner.set(contract);
    }

    /// Replaces the owner.
    pub fn set_owner(&mut self, contract: Address) {
        self.owner.set(contract);
    }

    /// The owner.
    pub fn owner(&self) -> Address {
        self.owner.get_or_revert_with(ExecutionError::UnwrapError)
    }

    /// The stored value.
    pub fn value(&self) -> u32 {
        self.value.get_or_default()
    }

    /// Adds both arguments to the stored value and returns it.
    pub fn add(&mut self, exec_env: u32, env_rc: u32) -> u32 {
        let value = self.value() + exec_env + env_rc;
        self.value.set(value);
        value
    }

    /// The attached value plus `exec_env`.
    #[odra(payable)]
    pub fn deposit(&mut self, exec_env: u32) -> U512 {
        self.env().attached_value() + U512::from(exec_env)
    }

    /// Combines the arguments, so that each of them must arrive in its place.
    pub fn echo(&self, named_args: u32, result: u32, call_def: u32, contract_env: u32) -> u32 {
        named_args * 1000 + result * 100 + call_def * 10 + contract_env
    }

    /// Stores `named_args` minus `contract`.
    #[odra(non_reentrant)]
    pub fn guarded(&mut self, named_args: u32, contract: u32) -> u32 {
        self.value.set(named_args - contract);
        self.value()
    }

    /// Runs on the host only.
    #[odra(offchain)]
    pub fn offchain_sum(&self, contract: u32, exec_env: u32, named_args: u32) -> u32 {
        self.value() + contract + exec_env + named_args
    }

    /// Calls [ArgNamesExt::echo] of the contract at `contract`.
    pub fn echo_of(&self, contract: Address, named_args: u32, result: u32) -> u32 {
        ArgNamesExtContractRef::new(self.env(), contract).echo(named_args, result, 0, 1)
    }
}

/// The interface of [ArgNames] seen as an external contract.
#[odra::external_contract]
pub trait ArgNamesExt {
    /// See [ArgNames::echo].
    fn echo(&self, named_args: u32, result: u32, call_def: u32, contract_env: u32) -> u32;
    /// See [ArgNames::add].
    fn add(&mut self, exec_env: u32, env_rc: u32) -> u32;
}

#[cfg(test)]
mod tests {
    use super::{ArgNames, ArgNamesExtHostRef, ArgNamesInitArgs};
    use odra::casper_types::U512;
    use odra::host::{Deployer, HostRef};
    use odra::prelude::Addressable;

    #[test]
    fn arguments_named_like_the_generated_code_work() {
        let env = odra_test::env();
        let (alice, bob) = (env.get_account(1), env.get_account(2));
        let mut contract = ArgNames::deploy(
            &env,
            ArgNamesInitArgs {
                env: 5,
                contract: alice
            }
        );
        assert_eq!(contract.value(), 5);
        assert_eq!(contract.owner(), alice);

        contract.set_owner(bob);
        assert_eq!(contract.owner(), bob);

        assert_eq!(contract.add(10, 20), 35);
        assert_eq!(contract.value(), 35);

        assert_eq!(
            contract.with_tokens(U512::from(100)).deposit(7),
            U512::from(107)
        );
        assert_eq!(contract.echo(1, 2, 3, 4), 1234);
        assert_eq!(contract.guarded(50, 8), 42);
        assert_eq!(contract.offchain_sum(1, 2, 3), 48);
    }

    #[test]
    fn external_contract_arguments_named_like_the_generated_code_work() {
        let env = odra_test::env();
        let init = |value| ArgNamesInitArgs {
            env: value,
            contract: env.get_account(0)
        };
        let caller = ArgNames::deploy(&env, init(0));
        let callee = ArgNames::deploy(&env, init(1));

        assert_eq!(caller.echo_of(callee.address(), 9, 8), 9801);

        let mut external = ArgNamesExtHostRef::new(callee.address(), env.clone());
        assert_eq!(external.echo(4, 3, 2, 1), 4321);
        assert_eq!(external.add(2, 3), 6);
        assert_eq!(callee.value(), 6);
    }
}
