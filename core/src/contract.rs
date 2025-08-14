/// The contract trait.
pub trait OdraContract {
    /// The host reference type.
    #[cfg(any(not(target_arch = "wasm32"), feature = "client"))]
    type HostRef: crate::host::HostRef
        + crate::host::EntryPointsCallerProvider
        + crate::contract_def::HasIdent;
    /// The contract reference type.
    type ContractRef: crate::contract_env::ContractRef;
    /// The init args type.
    #[cfg(any(not(target_arch = "wasm32"), feature = "client"))]
    type InitArgs: crate::host::InitArgs;

    /// The upgrade args type.
    #[cfg(any(not(target_arch = "wasm32"), feature = "client"))]
    type UpgradeArgs: crate::host::UpgradeArgs;
}
