//! Utility functions that allow to write more compact tests.

use crate::types::{
    odra_types::{self, EventData},
    Address, FromBytes, OdraType,
};

/// Gets the nth event emitted by the contract at `address`.
///
/// If the passed index is out of bounds, or a deserialization error occurs,
/// an error is returned.
pub fn get_event<T: OdraType>(
    contract_address: Address,
    at: i32,
) -> Result<T, odra_types::event::EventError> {
    let event: EventData = crate::test_env::get_event(contract_address, at)?;
    match T::from_bytes(&event) {
        Ok(res) => Ok(res.0),
        // TODO: Handle error conversion.
        Err(_) => Err(odra_types::event::EventError::Parsing),
    }
}

/// Gets the name of the nth event emitted by the contract at `address`.
///
/// If the passed index is out of bounds, or a deserialization error occurs,
/// an error is returned.
pub fn get_event_name(
    contract_address: Address,
    at: i32,
) -> Result<String, odra_types::event::EventError> {
    let event: EventData = crate::test_env::get_event(contract_address, at)?;
    let (event_name, _): (String, _) =
        FromBytes::from_vec(event).map_err(|_| odra_types::event::EventError::Parsing)?;
    Ok(event_name)
}

/// A macro that simplifies events testing.
///
/// It allows testing the presence of multiple events at once.
/// Two approaches are available, only the type of event,
/// or the full event can be passed, but not mixed.
///
/// The events must be passed in the natural order, from the first emitted events.
///
/// # Example
/// ```ignore
/// use odra::odra_types::event::Event;
///
/// #[derive(odra::Event, PartialEq, Eq, Debug)]
/// struct SetValue {
///     pub value: u32
/// }
///
/// #[derive(odra::Event, PartialEq, Eq, Debug)]
/// struct GetValue {
///     pub value: u32
/// }
///
/// // Contract initialization goes here
/// # let contract = ...;
///
/// SetValue { value: 1 }.emit();
/// SetValue { value: 42 }.emit();
/// SetValue { value: 8 }.emit();
///
/// // Assert the type only
/// odra::assert_events!(contract, SetValue, SetValue, GetValue);
///
/// // Assert the full event
/// odra::assert_events!(
///     contract,
///     SetValue { value: 1 },
///     SetValue { value: 42 },
///     GetValue { value: 8 }
/// );
/// ```
#[macro_export]
macro_rules! assert_events {
    ($contract:ident, $ ( $event_ty:ty ),+ ) => {
        let mut __idx = -1;
        $(
            __idx -= 1;
            let _ = stringify!($event_ty);
        )+
        $(
            __idx += 1;
            let __ev = odra::test_utils::get_event::<$event_ty>($contract.address(), __idx).unwrap();
            let __name = stringify!($event_ty).to_string();
            let __name = __name.split("::").last().unwrap();
            assert_eq!(
                <$event_ty as odra::types::event::Event>::name(&__ev), __name
            );
        )+
    };
    ($contract:ident, $ ( $event:expr ),+ ) => {
        let mut __idx = -1;
        $(
            __idx -= 1;
            let _ = $event;
        )+
        $(
            __idx += 1;
            let __ev = odra::test_utils::get_event(&$contract.address(), __idx).unwrap();
            assert_eq!(
                $event, __ev
            );
        )+
    };
}
