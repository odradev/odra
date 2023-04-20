use crate::attrs::partition_attributes;

/// Odra module struct.
///
/// Wraps up [syn::ItemStruct].
pub struct ModuleStruct {
    pub is_instantiable: bool,
    pub item: syn::ItemStruct
}

impl From<syn::ItemStruct> for ModuleStruct {
    fn from(item: syn::ItemStruct) -> Self {
        let (_, other_attrs) = partition_attributes(item.attrs).unwrap();
        Self {
            is_instantiable: true,
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item
            }
        }
    }
}
