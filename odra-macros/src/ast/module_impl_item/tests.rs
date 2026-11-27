use super::*;
use crate::test_utils::{self, mock};

/// The impl block expands to the user's code and every generated part, in the order of
/// [ModuleImplItem]: each part's content is pinned by its own tests, this pins that none is
/// left out and that the module check comes before the rest.
#[test]
fn the_expansion_is_every_part_in_order() {
    let ir = mock::module_factory_on();
    let module_ident = ir.module_ident();
    let parts = [
        ir.self_code(),
        quote!(impl odra::module::IsModule for #module_ident {}),
        HasEntrypointsImplItem::try_from(&ir).unwrap().into_token_stream(),
        RefItem::try_from(&ir).unwrap().into_token_stream(),
        TestPartsItem::try_from(&ir).unwrap().into_token_stream(),
        TestPartsReexportItem::try_from(&ir).unwrap().into_token_stream(),
        ModuleExecPartsItem::try_from(&ir).unwrap().into_token_stream(),
        OdraModuleCfgItem::<ModuleWasmPartsItem>::try_from(&ir)
            .unwrap()
            .into_token_stream(),
        ContractItem::try_from(&ir).unwrap().into_token_stream(),
        OdraModuleCfgItem::<BlueprintItem>::try_from(&ir)
            .unwrap()
            .into_token_stream(),
        SchemaEntrypointsItem::try_from(&ir).unwrap().into_token_stream(),
        SchemaCustomTypesItem::try_from(&ir).unwrap().into_token_stream(),
        FactoryModuleImplItem::try_from(&ir).unwrap().into_token_stream()
    ];
    // Every part emits something for this module, so a dropped one changes the expansion.
    assert!(parts.iter().all(|part| !part.is_empty()));
    let actual = ModuleImplItem::try_from(&ir).unwrap();
    test_utils::assert_eq(actual, quote!(#(#parts)*));
}
