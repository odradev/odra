/// The contract trait.
pub trait OdraContract {
    /// The host reference type.
    #[cfg(not(target_arch = "wasm32"))]
    type HostRef: crate::host::HostRef
        + crate::host::EntryPointsCallerProvider
        + crate::contract_def::HasIdent;
    /// The contract reference type.
    type ContractRef: crate::ContractRef;
    /// The init args type.
    #[cfg(not(target_arch = "wasm32"))]
    type InitArgs: crate::host::InitArgs;
}
