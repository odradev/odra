pub struct ReentrancyGuard;

impl ReentrancyGuard {

    pub fn reentrancy_guard_entered() -> bool {
        #[cfg(feature = "casper")] {
            odra::contract_env::get_var("__reentrancy_guard").unwrap_or_default()
        }
        #[cfg(feature = "mock-vm")] {
            false
        }
    }
}