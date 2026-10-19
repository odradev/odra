//! Odra's contracts build script.

/// Uses the ENV variables `ODRA_MODULE` and `ODRA_BACKEND` to set the `odra_module` and
/// `odra_backend` cfg flags.
pub fn main() {
    odra_build::build();
}
