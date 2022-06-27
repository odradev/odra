macro_rules! as_ref_for_contract_impl_generator {
    ($struct_ident:ident) => {
        impl ::core::convert::AsRef<odra_ir::contract_item::contract_impl::ContractImpl>
            for $struct_ident<'_>
        {
            fn as_ref(&self) -> &odra_ir::contract_item::contract_impl::ContractImpl {
                self.contract
            }
        }
    };
}

pub mod generator;

pub trait OdraPoet: Sized {
    type Poet: From<Self> + GenerateCode;
}

impl<'a> OdraPoet for &'a odra_ir::ContractImpl {
    type Poet = generator::ContractImpl<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::ContractStruct {
    type Poet = generator::ContractStruct<'a>;
}

pub trait GenerateCode {
    fn generate_code(&self) -> proc_macro2::TokenStream;
}

pub trait OdraPoetUsingImpl: AsRef<odra_ir::ContractImpl> {
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::ContractImpl>;
}

impl<T> OdraPoetUsingImpl for T
where
    T: AsRef<odra_ir::ContractImpl>,
{
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::ContractImpl>,
    {
        <G as GenerateCode>::generate_code(&G::from(<Self as AsRef<odra_ir::ContractImpl>>::as_ref(self)))
    }
}

pub fn generate_code<T>(entity: T) -> proc_macro2::TokenStream
where
    T: OdraPoet,
{
    <T as OdraPoet>::Poet::from(entity).generate_code()
}
