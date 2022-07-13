macro_rules! as_ref_for_contract_impl_generator {
    ($struct_ident:ident) => {
        impl ::core::convert::AsRef<odra_ir::module_item::module_impl::ModuleImpl>
            for $struct_ident<'_>
        {
            fn as_ref(&self) -> &odra_ir::module_item::module_impl::ModuleImpl {
                self.contract
            }
        }
    };
}

pub mod generator;

pub trait OdraPoet: Sized {
    type Poet: From<Self> + GenerateCode;
}

impl<'a> OdraPoet for &'a odra_ir::ModuleImpl {
    type Poet = generator::ModuleImpl<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::ModuleStruct {
    type Poet = generator::ModuleStruct<'a>;
}

pub trait GenerateCode {
    fn generate_code(&self) -> proc_macro2::TokenStream;
}

pub trait OdraPoetUsingImpl: AsRef<odra_ir::ModuleImpl> {
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::ModuleImpl>;
}

impl<T> OdraPoetUsingImpl for T
where
    T: AsRef<odra_ir::ModuleImpl>,
{
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::ModuleImpl>,
    {
        <G as GenerateCode>::generate_code(&G::from(<Self as AsRef<odra_ir::ModuleImpl>>::as_ref(
            self,
        )))
    }
}

pub fn generate_code<T>(entity: T) -> proc_macro2::TokenStream
where
    T: OdraPoet,
{
    <T as OdraPoet>::Poet::from(entity).generate_code()
}
