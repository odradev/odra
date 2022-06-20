use std::cell::RefCell;

use odra_types::Address;

pub(crate) fn next_address() -> Address {
    thread_local! {
        static COUNTER: RefCell<u32> = RefCell::new(0)
    }
    COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
        let counter = *counter.borrow();
        Address::new(&counter.to_be_bytes())
    })
}

pub(crate) fn push_address(address: &Address) {
    odra_mock_vm::borrow_mut_env().push_address(address)
}

pub(crate) fn pop_address() {
    odra_mock_vm::borrow_mut_env().pop_address()
}
