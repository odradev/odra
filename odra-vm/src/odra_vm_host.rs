use crate::odra_vm_contract_env::OdraVmContractEnv;
use crate::OdraVm;
use odra_core::casper_types::{bytesrepr::Bytes, PublicKey, RuntimeArgs, U512};
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{
    host::{HostContext, HostEnv},
    CallDef, ContractContext, ContractEnv
};
use odra_core::{prelude::*, OdraResult};
use odra_core::{Address, OdraError, VmError};
use odra_core::{EventError, GasReport};

/// HostContext utilizing the Odra in-memory virtual machine.
pub struct OdraVmHost {
    vm: Rc<RefCell<OdraVm>>,
    contract_env: Rc<ContractEnv>
}

impl HostContext for OdraVmHost {
    fn set_caller(&self, caller: Address) {
        self.vm.borrow().set_caller(caller)
    }

    fn set_gas(&self, gas: u64) {
        // Set gas does nothing in this context
    }

    fn caller(&self) -> Address {
        *self.vm.borrow().callstack_tip().address()
    }

    fn get_account(&self, index: usize) -> Address {
        self.vm.borrow().get_account(index)
    }

    fn balance_of(&self, address: &Address) -> U512 {
        self.vm.borrow().balance_of(address)
    }

    fn advance_block_time(&self, time_diff: u64) {
        self.vm.borrow().advance_block_time_by(time_diff)
    }

    fn block_time(&self) -> u64 {
        self.vm.borrow().get_block_time()
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        self.vm.borrow().get_event(contract_address, index)
    }

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        self.vm.borrow().get_events_count(contract_address)
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        _use_proxy: bool
    ) -> OdraResult<Bytes> {
        let mut opt_result: Option<Bytes> = None;
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            opt_result = Some(self.vm.borrow().call_contract(*address, call_def));
        }));

        match opt_result {
            Some(result) => Ok(result),
            None => {
                let error = self.vm.borrow().error();
                Err(error.unwrap_or(OdraError::VmError(VmError::Panic)))
            }
        }
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let address = self
            .vm
            .borrow()
            .register_contract(name, entry_points_caller.clone());

        if entry_points_caller
            .entry_points()
            .iter()
            .any(|ep| ep.name == "init")
        {
            self.call_contract(
                &address,
                CallDef::new(String::from("init"), true, init_args),
                false
            )?;
        }

        Ok(address)
    }

    fn register_contract(&self, address: Address, contract_name: String, entry_points_caller: EntryPointsCaller) {
        panic!("register_contract is not supported for OdraVM");
    }

    fn contract_env(&self) -> ContractEnv {
        (*self.contract_env).clone()
    }

    fn gas_report(&self) -> GasReport {
        // For OdraVM there is no gas, so nothing to report.c
        GasReport::new()
    }

    fn last_call_gas_cost(&self) -> u64 {
        // For OdraVM there is no gas, so nothing to return.
        0
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        self.vm.borrow().sign_message(message, address)
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        self.vm.borrow().public_key(address)
    }

    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        let caller = self.caller();
        self.vm
            .borrow()
            .checked_transfer_tokens(&caller, &to, &amount)
    }
}

impl OdraVmHost {
    /// Creates a new `OdraVmHost` instance.
    pub fn new(vm: Rc<RefCell<OdraVm>>) -> Rc<RefCell<Self>> {
        let contract_env = Rc::new(ContractEnv::new(0, OdraVmContractEnv::new(vm.clone())));
        Rc::new(RefCell::new(Self { vm, contract_env }))
    }
}
