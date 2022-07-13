cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_vm::test_utils::*;
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
