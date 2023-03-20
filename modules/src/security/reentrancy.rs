pub struct ReentrancyGuard;

impl ReentrancyGuard {
    pub fn reentrancy_guard_entered() -> bool {
        odra::contract_env::get_var("__reentrancy_guard").unwrap_or_default()
    }
}
