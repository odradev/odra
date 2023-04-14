use crate::attrs::partition_attributes;

use super::ModuleEvents;

/// Odra module struct.
///
/// Wraps up [syn::ItemStruct].
pub struct ModuleStruct {
    pub is_instantiable: bool,
    pub item: syn::ItemStruct,
    pub events: ModuleEvents
}

impl ModuleStruct {
    pub fn with_events(mut self, events: ModuleEvents) -> Self {
        self.events = events;
        self
    }
}

impl From<syn::ItemStruct> for ModuleStruct {
    fn from(item: syn::ItemStruct) -> Self {
        let (_, other_attrs) = partition_attributes(item.attrs).unwrap();
        Self {
            is_instantiable: true,
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item
            },
            events: Default::default()
        }
    }
}
