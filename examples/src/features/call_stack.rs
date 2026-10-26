//! This example shows how a contract looks past its immediate caller at the whole call stack.
use odra::prelude::*;

/// Reports who called it, who called the caller, and the full call stack.
#[odra::module]
pub struct CallStackProbe;

#[odra::module]
impl CallStackProbe {
    /// Returns the immediate caller, the caller of the caller (if any) and the call stack from
    /// the initiating account to this contract.
    pub fn inspect(&self) -> (Address, Option<Address>, Vec<Address>) {
        let env = self.env();
        (env.caller(), env.nth_caller(1), env.call_stack())
    }

    /// The same as [inspect](Self::inspect), executed on the host instead of deployed.
    #[odra(offchain)]
    pub fn inspect_offchain(&self) -> (Address, Option<Address>, Vec<Address>) {
        self.inspect()
    }
}

/// Puts one more contract frame between the account and the probe.
#[odra::module]
pub struct CallStackRelay {
    probe: External<CallStackProbeContractRef>
}

#[odra::module]
impl CallStackRelay {
    /// Initializes the contract with the address of the probe.
    pub fn init(&mut self, probe: Address) {
        self.probe.set(probe);
    }

    /// Calls the probe and passes its answer through.
    pub fn relay(&self) -> (Address, Option<Address>, Vec<Address>) {
        self.probe.inspect()
    }

    /// Calls the probe from the host, like [relay](Self::relay) does on chain.
    #[odra(offchain)]
    pub fn relay_offchain(&self) -> (Address, Option<Address>, Vec<Address>) {
        self.probe.inspect()
    }
}

#[cfg(test)]
mod tests {
    use super::{CallStackProbe, CallStackRelay, CallStackRelayInitArgs};
    use odra::host::{Deployer, NoArgs};
    use odra::prelude::*;

    #[test]
    fn call_stack_is_visible_through_intermediary_contracts() {
        let env = odra_test::env();
        let probe = CallStackProbe::deploy(&env, NoArgs);
        let relay = CallStackRelay::deploy(
            &env,
            CallStackRelayInitArgs {
                probe: probe.address()
            }
        );
        let alice = env.get_account(1);
        env.set_caller(alice);

        // Called directly: the account is the caller and there is nobody behind it.
        assert_eq!(probe.inspect(), (alice, None, vec![alice, probe.address()]));

        // Called through the relay: the relay is the caller, the account is behind it.
        assert_eq!(
            relay.relay(),
            (
                relay.address(),
                Some(alice),
                vec![alice, relay.address(), probe.address()]
            )
        );
    }

    #[test]
    fn offchain_functions_see_the_same_call_stack() {
        let env = odra_test::env();
        let probe = CallStackProbe::deploy(&env, NoArgs);
        let relay = CallStackRelay::deploy(
            &env,
            CallStackRelayInitArgs {
                probe: probe.address()
            }
        );
        let alice = env.get_account(1);
        env.set_caller(alice);

        assert_eq!(
            probe.inspect_offchain(),
            (alice, None, vec![alice, probe.address()])
        );
        assert_eq!(
            relay.relay_offchain(),
            (
                relay.address(),
                Some(alice),
                vec![alice, relay.address(), probe.address()]
            )
        );
    }
}
