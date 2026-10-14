/// The contract trait.
pub trait OdraContract: crate::contract_def::HasEvents {
    /// The host reference type.
    #[cfg(not(target_arch = "wasm32"))]
    type HostRef: crate::host::HostRef
        + crate::host::EntryPointsCallerProvider
        + crate::contract_def::HasIdent;
    /// The contract reference type.
    type ContractRef: crate::contract_env::ContractRef;
    /// The init args type.
    #[cfg(not(target_arch = "wasm32"))]
    type InitArgs: crate::host::InitArgs;

    /// The upgrade args type.
    #[cfg(not(target_arch = "wasm32"))]
    type UpgradeArgs: crate::host::UpgradeArgs;

    /// The events that must be emittable for the contract to be deployed: its own (see
    /// [HasEvents::events](crate::contract_def::HasEvents::events)) and, for a factory, those of
    /// the contracts it creates, whose code it carries. Checked when the contract is deployed or
    /// upgraded, see [assert_events_can_be_emitted](crate::contract_def::assert_events_can_be_emitted).
    #[cfg(not(target_arch = "wasm32"))]
    fn contract_events() -> crate::prelude::Vec<crate::contract_def::Event> {
        Self::events()
    }
}
