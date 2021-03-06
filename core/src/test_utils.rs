use odra_types::{EventData, FromBytes, Address, event::Error as EventError};

pub fn get_event<T>(contract_address: &Address, at: i32) -> Result<T, odra_types::event::Error>
where
    T: FromBytes<Item = T, Error = odra_types::event::Error>,
{
    let event: EventData = crate::TestEnv::get_event(contract_address, at)?;
    match T::deserialize(event) {
        Ok(res) => Ok(res.0),
        Err(err) => Err(err),
    }
}

#[macro_export]
macro_rules! assert_events {
    ($contract:ident, $ ( $event_ty:ty ),+ ) => {
        let mut __idx = 0;
        $(
            __idx -= 1;
            let ev = odra::test_utils::get_event::<$event_ty>(&$contract.address(), __idx).unwrap();
            assert_eq!(
                <$event_ty as odra::types::event::Event>::name(&ev), stringify!($event_ty).to_string()
            );
        )+
    };
    ($contract:ident, $ ( $event:expr ),+ ) => {
        let mut __idx = 0;
        $(
            __idx -= 1;
            let ev = odra::test_utils::get_event(&$contract.address(), __idx).unwrap();
            assert_eq!(
                $event, ev
            );
        )+
    };
}

pub fn event_absolute_position(len: u32, index: i32) -> Result<u32, EventError> {
    if index.is_negative() {
        let abs_idx = index.wrapping_abs() as u32;
        if abs_idx > len {
            return Err(EventError::IndexOutOfBounds);
        }
        Ok(len - abs_idx)
    } else {
        if index as u32 >= len {
            return Err(EventError::IndexOutOfBounds);
        }
        Ok(index as u32)
    }
}
