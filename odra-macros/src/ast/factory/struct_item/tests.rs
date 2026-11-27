use super::*;
use crate::test_utils::{self, mock};

/// The factory struct expands to the struct itself and every generated part, in the order of
/// [FactoryModuleStructItem]; each part's content is pinned by its own tests.
#[test]
fn the_expansion_is_every_part_in_order() {
    let ir = mock::factory_module_definition();
    let parts = [
        ModuleDefItem::try_from(&ir).unwrap().into_token_stream(),
        ModuleModItem::try_from(&ir).unwrap().into_token_stream(),
        HasIdentImplItem::try_from(&ir).unwrap().into_token_stream(),
        FactoryEventItem::try_from(&ir).unwrap().into_token_stream(),
        FactoryHasEventsImplItem::try_from(&ir).unwrap().into_token_stream(),
        OdraModuleCfgItem::<SchemaItem>::try_from(&ir)
            .unwrap()
            .into_token_stream(),
        FactorySchemaEventsItem::try_from(&ir).unwrap().into_token_stream(),
        FactorySchemaErrorsItem::try_from(&ir).unwrap().into_token_stream(),
        SchemaStorageLayoutItem::try_from(&ir).unwrap().into_token_stream()
    ];
    assert!(parts.iter().all(|part| !part.is_empty()));
    let actual = FactoryModuleStructItem::try_from(&ir).unwrap();
    test_utils::assert_eq(actual, quote!(#(#parts)*));
}
