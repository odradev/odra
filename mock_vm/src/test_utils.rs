use std::fmt::Debug;

use odra_mock_vm_macros::event_test;
use odra_types::{Address, EventData, FromBytes, ToBytes};

use crate::borrow_env;

#[event_test]
pub fn get_event<T>(contract_address: &Address, at: i32) -> Result<T, odra_types::event::Error>
where
    T: FromBytes<Item = T, Error = odra_types::event::Error>,
{
    let events = events(contract_address);
    let len = events.len();
    let idx = index_to_usize(len, at)?;

    let data = events.get(idx).unwrap().clone();
    match T::deserialize(data) {
        Ok(res) => Ok(res.0),
        Err(err) => Err(err),
    }
}

#[event_test]
pub fn assert_event_emitted<T>(contract_address: &Address, event: T)
where
    T: ToBytes<Error = odra_types::event::Error> + Debug,
{
    let events = events(contract_address);
    let event_data = event.serialize().unwrap();
    for event in events.iter() {
        if event == &event_data {
            return;
        }
    }

    panic!(
        r#"assertion failed: event not found
        expected: `{:?}`"#,
        event
    );
}

#[event_test]
pub fn assert_event_type_emitted(contract_address: &Address, event_name: &str) {
    let events = events(contract_address);
    for event in events.iter() {
        if extract_event_name(event.as_slice()) == Ok(event_name.to_string()) {
            return;
        }
    }

    panic!(
        r#"assertion failed: event of the specified type not found
        expected: `{:?}`"#,
        event_name
    );
}

pub fn assert_event<T>(contract_address: &Address, at: i32, event: T)
where
    T: FromBytes<Item = T, Error = odra_types::event::Error> + Debug + PartialEq,
{
    let expected_event = get_event::<T>(contract_address, at).unwrap();
    assert_eq!(expected_event, event)
}

#[event_test]
pub fn assert_event_type(contract_address: &Address, event_name: &str, at: i32) {
    let events = events(contract_address);
    let len = events.len();
    let idx = index_to_usize(len, at).unwrap();

    assert_eq!(
        extract_event_name(events.get(idx).unwrap()),
        Ok(event_name.to_string())
    );
}

#[event_test]
pub fn assert_event_type_not_emitted(contract_address: &Address, event_name: &str) {
    let events = events(contract_address);
    for event in events.iter() {
        if extract_event_name(event.as_slice()) == Ok(event_name.to_string()) {
            panic!(
                r#"assertion failed: unexpected event of type {} found"#,
                event_name
            );
        }
    }
}

#[event_test]
pub fn assert_event_not_emitted<T>(contract_address: &Address, event: T)
where
    T: ToBytes<Error = odra_types::event::Error> + Debug + PartialEq,
{
    let events = events(contract_address);
    let event_data = event.serialize().unwrap();
    for event in events.iter() {
        if event == &event_data {
            panic!(r#"assertion failed: unexpected event {:?} found"#, event);
        }
    }
}

fn extract_event_name(event_data: &[u8]) -> Result<String, odra_types::event::Error> {
    let (name, _): (String, _) = odra_types::bytesrepr::FromBytes::from_bytes(event_data)?;
    Ok(name)
}

fn index_to_usize(len: usize, index: i32) -> Result<usize, odra_types::event::Error> {
    if index.is_negative() {
        let abs_idx = index.wrapping_abs() as usize;
        if abs_idx > len {
            return Err(odra_types::event::Error::IndexOutOfBounds);
        }
        Ok(len - abs_idx)
    } else {
        if index as usize >= len {
            return Err(odra_types::event::Error::IndexOutOfBounds);
        }
        Ok(index as usize)
    }
}

fn events(contract_address: &Address) -> Vec<EventData> {
    borrow_env().events(contract_address)
}

#[cfg(test)]
mod test {
    use odra_types::{EventData, ToBytes};

    use super::{
        test_assert_event_emitted, test_assert_event_not_emitted, test_assert_event_type,
        test_assert_event_type_emitted, test_assert_event_type_not_emitted, test_get_event,
    };

    #[derive(Debug, PartialEq)]
    struct Set {
        value: u32,
    }

    impl odra_types::ToBytes for Set {
        type Error = odra_types::event::Error;
        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            core::result::Result::Ok(<Self as odra_types::bytesrepr::ToBytes>::to_bytes(self)?)
        }
    }
    impl odra_types::FromBytes for Set {
        type Error = odra_types::event::Error;
        type Item = Self;
        fn deserialize(data: Vec<u8>) -> Result<(Self::Item, Vec<u8>), Self::Error> {
            let (event_name, bytes): (String, _) =
                odra_types::bytesrepr::FromBytes::from_vec(data)?;
            if &event_name != "Set" {
                return core::result::Result::Err(odra_types::event::Error::UnexpectedType(
                    event_name,
                ));
            }
            let (value, bytes) = odra_types::bytesrepr::FromBytes::from_vec(bytes)?;
            let value = Set { value };
            Ok((value, bytes))
        }
    }
    impl odra_types::bytesrepr::ToBytes for Set {
        fn serialized_length(&self) -> usize {
            let mut size = 0;
            size += "Set".serialized_length();
            size += self.value.serialized_length();
            return size;
        }
        fn to_bytes(&self) -> Result<Vec<u8>, odra_types::bytesrepr::Error> {
            let mut vec = Vec::with_capacity(self.serialized_length());
            vec.append(&mut "Set".to_bytes()?);
            vec.extend(self.value.to_bytes()?);
            Ok(vec)
        }
    }

    #[derive(Debug, PartialEq)]
    struct Get {
        value: String,
    }

    impl odra_types::ToBytes for Get {
        type Error = odra_types::event::Error;
        fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
            core::result::Result::Ok(<Self as odra_types::bytesrepr::ToBytes>::to_bytes(self)?)
        }
    }
    impl odra_types::FromBytes for Get {
        type Error = odra_types::event::Error;
        type Item = Self;
        fn deserialize(data: Vec<u8>) -> Result<(Self::Item, Vec<u8>), Self::Error> {
            let (event_name, bytes): (String, _) =
                odra_types::bytesrepr::FromBytes::from_vec(data)?;
            if &event_name != "Get" {
                return core::result::Result::Err(odra_types::event::Error::UnexpectedType(
                    event_name,
                ));
            }
            let (value, bytes) = odra_types::bytesrepr::FromBytes::from_vec(bytes)?;
            let value = Get { value };
            Ok((value, bytes))
        }
    }
    impl odra_types::bytesrepr::ToBytes for Get {
        fn serialized_length(&self) -> usize {
            let mut size = 0;
            size += "Get".serialized_length();
            size += self.value.serialized_length();
            return size;
        }
        fn to_bytes(&self) -> Result<Vec<u8>, odra_types::bytesrepr::Error> {
            let mut vec = Vec::with_capacity(self.serialized_length());
            vec.append(&mut "Get".to_bytes()?);
            vec.extend(self.value.to_bytes()?);
            Ok(vec)
        }
    }

    fn stub_events() -> Vec<EventData> {
        vec![
            Set { value: 1 }.serialize().unwrap(),
            Set { value: 10 }.serialize().unwrap(),
            Set { value: 100 }.serialize().unwrap(),
            Get {
                value: String::from("Elo"),
            }
            .serialize()
            .unwrap(),
            vec![1, 2, 3, 4, 1],
        ]
    }

    #[test]
    fn test_getting_events() {
        assert_eq!(
            test_get_event::<Set>(stub_events(), 0),
            Ok(Set { value: 1 })
        );
        assert_eq!(
            test_get_event::<Get>(stub_events(), 3),
            Ok(Get {
                value: String::from("Elo")
            })
        );
        assert_eq!(
            test_get_event::<Set>(stub_events(), -4),
            Ok(Set { value: 10 })
        );
        assert_eq!(
            test_get_event::<Get>(stub_events(), -2),
            Ok(Get {
                value: String::from("Elo")
            })
        );
    }

    #[test]
    fn getting_events_errors() {
        assert_eq!(
            test_get_event::<Set>(stub_events(), 5),
            Err(odra_types::event::Error::IndexOutOfBounds)
        );
        assert_eq!(
            test_get_event::<Set>(stub_events(), -10),
            Err(odra_types::event::Error::IndexOutOfBounds)
        );
        assert_eq!(
            test_get_event::<Get>(stub_events(), 0), // Set { value: 1 }
            Err(odra_types::event::Error::UnexpectedType("Set".to_string()))
        );
        assert_eq!(
            test_get_event::<Get>(stub_events(), 4), // Unstructured bytes vec
            Err(odra_types::event::Error::Parsing)
        );
    }

    #[test]
    fn assert_event_emitted() {
        test_assert_event_emitted(stub_events(), Set { value: 1 });
        test_assert_event_emitted(
            stub_events(),
            Get {
                value: String::from("Elo"),
            },
        );

        test_assert_event_type_emitted(stub_events(), "Set");
        test_assert_event_type_emitted(stub_events(), "Get");
    }

    #[test]
    #[should_panic]
    fn fail_assert_event_emitted() {
        test_assert_event_emitted(stub_events(), Set { value: 111 });
    }

    #[test]
    #[should_panic]
    fn fail_assert_event_type_emitted() {
        test_assert_event_type_emitted(stub_events(), "Event");
    }

    #[test]
    fn assert_event_type() {
        test_assert_event_type(stub_events(), "Set", 0);
    }

    #[test]
    #[should_panic]
    fn assert_event_type_failure() {
        test_assert_event_type(stub_events(), "Get", 0);
    }

    #[test]
    fn assert_event_type_not_emitted() {
        test_assert_event_type_not_emitted(stub_events(), "Ev");
    }

    #[test]
    #[should_panic]
    fn assert_event_type_not_emitted_fail() {
        test_assert_event_type_not_emitted(stub_events(), "Set");
    }

    #[test]
    fn assert_event_not_emitted() {
        test_assert_event_not_emitted(stub_events(), Set { value: 123 });
    }

    #[test]
    #[should_panic]
    fn assert_event_not_emitted_fail() {
        test_assert_event_not_emitted(stub_events(), Set { value: 1 });
    }
}
