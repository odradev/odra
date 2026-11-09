use quote::ToTokens;

#[test]
fn odra_module_accepts_the_bare_and_the_crate_qualified_name() {
    let attr = super::odra_module_of_crate("odra_modules", "Erc20");
    assert_eq!(
        attr.to_token_stream().to_string(),
        quote::quote!(#[cfg(any(odra_module = "Erc20", odra_module = "odra_modules::Erc20"))])
            .to_string()
    );
}

#[test]
fn current_crate_name_falls_back_when_the_env_var_is_missing() {
    // `CARGO_CRATE_NAME` is set by cargo for every rustc invocation, but not for a plain
    // test binary run, so the fallback is what the expansion tests observe.
    assert_eq!(super::current_crate_name(), "unknown_crate");
}
