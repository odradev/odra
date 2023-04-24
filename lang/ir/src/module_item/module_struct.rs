use crate::attrs::partition_attributes;
use anyhow::Result;

use super::ModuleConfiguration;

/// Odra module struct.
///
/// Wraps up [syn::ItemStruct].
pub struct ModuleStruct {
    pub is_instantiable: bool,
    pub item: syn::ItemStruct,
    pub skip_instance: bool
}

impl ModuleStruct {
    pub fn with_config(mut self, config: ModuleConfiguration) -> Result<Self, syn::Error> {
        self.skip_instance = config.skip_instance;

        Ok(self)
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
            skip_instance: false
        }
    }
}
