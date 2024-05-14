use crate::ast::events_item::HasEventsImplItem;
use crate::ast::fn_utils::{FnItem, SingleArgFnItem};
use crate::ast::utils::ImplItem;
use crate::ir::{TypeIR, TypeKind};
use crate::utils;
use crate::utils::misc::AsBlock;
use derive_try_from_ref::TryFromRef;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, Token};
use syn::punctuated::Punctuated;
use crate::ast::schema::SchemaCustomTypeItem;

macro_rules! impl_from_ir {
    ($ty:path) => {
        impl TryFrom<&'_ TypeIR> for $ty {
            type Error = syn::Error;

            fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
                match ir.kind()? {
                    TypeKind::UnitEnum { variants } => Self::from_unit_enum(variants),
                    TypeKind::Enum { variants } => Self::from_enum(variants),
                    TypeKind::Struct { fields } => Self::from_struct(fields),
                }
            }
        }
    };
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(TypeIR)]
#[err(syn::Error)]
pub struct OdraTypeAttrItem {
    item: OdraTypeItem,
    schema: SchemaCustomTypeItem
}

#[derive(syn_derive::ToTokens)]
pub struct OdraTypeItem {
    attr: syn::Attribute,
    item: syn::Item,
    from_bytes_impl: FromBytesItem,
    to_bytes_impl: ToBytesItem,
    cl_type_impl: CLTypedItem,
    has_events_impl: HasEventsImplItem
}

impl TryFrom<&'_ TypeIR> for OdraTypeItem {
    type Error = syn::Error;
    fn try_from(input: &'_ TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            attr: utils::attr::common_derive_attr(),
            item: input.self_code().clone(),
            from_bytes_impl: input.try_into()?,
            to_bytes_impl: input.try_into()?,
            cl_type_impl: input.try_into()?,
            has_events_impl: input.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct FromBytesItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: FromBytesFnItem
}

impl TryFrom<&'_ TypeIR> for FromBytesItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::from_bytes(ir)?,
            brace_token: Default::default(),
            fn_item: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ToBytesItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: ToBytesFnItem,
    #[syn(in = brace_token)]
    serialized_length_item: SerializedLengthFnItem
}

impl TryFrom<&'_ TypeIR> for ToBytesItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_item: ImplItem::to_bytes(ir)?,
            brace_token: Default::default(),
            fn_item: ir.try_into()?,
            serialized_length_item: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct CLTypedItem {
    impl_item: ImplItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    fn_item: FnItem
}

impl TryFrom<&'_ TypeIR> for CLTypedItem {
    type Error = syn::Error;

    fn try_from(ir: &TypeIR) -> Result<Self, Self::Error> {
        let ret_ty_cl_type_any = match ir.kind()? {
            TypeKind::UnitEnum { variants: _ } => utils::ty::cl_type_u8(),
            TypeKind::Enum { variants: _ } => utils::ty::cl_type_any(),
            TypeKind::Struct { fields: _ } => utils::ty::cl_type_any(),
        }
        .as_block();
        let ty_cl_type = utils::ty::cl_type();
        Ok(Self {
            impl_item: ImplItem::cl_typed(ir)?,
            brace_token: Default::default(),
            fn_item: FnItem::new(
                &utils::ident::cl_type(),
                vec![],
                utils::misc::ret_ty(&ty_cl_type),
                ret_ty_cl_type_any
            )
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct FromBytesFnItem {
    fn_item: SingleArgFnItem
}

impl_from_ir!(FromBytesFnItem);

impl FromBytesFnItem {
    fn from_enum(variants: Vec<syn::Variant>)  -> syn::Result<Self> {
        let ident_bytes = utils::ident::bytes();
        let ident_from_bytes = utils::ident::from_bytes();
        let ident_result = utils::ident::result();
        let ty_u8 = utils::ty::u8();
        let from_bytes_expr = utils::expr::failable_from_bytes(&ident_bytes);

        let read_stmt: syn::Stmt =
            parse_quote!(let (#ident_result, #ident_bytes): (#ty_u8, _) = #from_bytes_expr;);
        let arms = variants
            .iter()
            .enumerate()
            .map(|(v_idx, v)| {
                let v_idx: u8 = v_idx as u8;
                let ident = &v.ident;
                let fields = variant_ident_vec(v);
                let deser = fields.iter()
                    .map(|f| quote::quote!(let (#f, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;))
                    .collect::<Vec<_>>();
                let code = match &v.fields {
                    syn::Fields::Unit => {
                        quote::quote!(Ok((Self::#ident, bytes)))
                    },
                    syn::Fields::Named(_) => {
                        quote::quote!(
                            #(#deser)*
                            Ok((Self::#ident { #(#fields,)* }, bytes))
                        )
                    },
                    syn::Fields::Unnamed(_) => {
                        quote::quote!(
                            #(#deser)*
                            Ok((Self::#ident(#(#fields,)*), bytes))
                        )
                    }
                };
                quote::quote!(#v_idx => { #code })
            })
            .collect::<Punctuated<_, Token![,]>>();

        let arg = Self::arg();
        let ret_ty = Self::ret_ty();
        let block = parse_quote!({
            #read_stmt
            match #ident_result {
                #arms 
                _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
            }
        });
        Ok(Self {
            fn_item: SingleArgFnItem::new(&ident_from_bytes, arg, ret_ty, block)
        })
    }

    fn from_unit_enum(variants: Vec<syn::Variant>) -> syn::Result<Self> {
        let ident_bytes = utils::ident::bytes();
        let ident_from_bytes = utils::ident::from_bytes();
        let ident_result = utils::ident::result();
        let ty_u8 = utils::ty::u8();
        let from_bytes_expr = utils::expr::failable_from_bytes(&ident_bytes);

        let read_stmt: syn::Stmt =
            parse_quote!(let (#ident_result, #ident_bytes): (#ty_u8, _) = #from_bytes_expr;);
        let deser = variants.iter()
            .map(|v|  {
                let i = &v.ident;
                let self_ty = match &v.fields {
                    syn::Fields::Unit => quote::quote!(Self::#i),
                    syn::Fields::Named(_) => quote::quote!(Self::#i { }),
                    syn::Fields::Unnamed(_) => quote::quote!(Self::#i())
                };
                quote::quote!(x if x == #self_ty as #ty_u8 => Ok((#self_ty, #ident_bytes)))
            })
            .collect::<Vec<_>>();
        let arg = Self::arg();
        let ret_ty = Self::ret_ty();
        let block = parse_quote!({
            #read_stmt
            match #ident_result {
                #(#deser,)*
                _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
            }
        });
        Ok(Self {
            fn_item: SingleArgFnItem::new(&ident_from_bytes, arg, ret_ty, block)
        })
    }

    fn from_struct(fields: Vec<(syn::Ident, syn::Type)>) -> syn::Result<Self> {
        let ident_bytes = utils::ident::bytes();
        let ident_from_bytes = utils::ident::from_bytes();

        let from_bytes_expr = utils::expr::failable_from_bytes(&ident_bytes);
        let fields = fields
            .into_iter()
            .map(|(i, _)| i)
            .collect::<syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>>();
        let deser = fields.iter()
            .map(|i| quote::quote!(let (#i, #ident_bytes) = #from_bytes_expr;))
            .collect::<Vec<_>>();
        let arg = Self::arg();
        let ret_ty = Self::ret_ty();
        let block = parse_quote!({
            #(#deser)*
            Ok((Self { #fields }, #ident_bytes))
        });

        Ok(Self {
            fn_item: SingleArgFnItem::new(&ident_from_bytes, arg, ret_ty, block)
        })
    }

    fn arg() -> syn::FnArg {
        let ident_bytes = utils::ident::bytes();
        let ty_bytes_slice = utils::ty::bytes_slice();
        parse_quote!(#ident_bytes: #ty_bytes_slice)
    }

    fn ret_ty() -> syn::ReturnType {
        let ty_bytes_slice = utils::ty::bytes_slice();
        let ty_self = utils::ty::_Self();
        let ty_ok = parse_quote!((#ty_self, #ty_bytes_slice));
        let ty_ret = utils::ty::bytes_result(&ty_ok);
        utils::misc::ret_ty(&ty_ret)
    }
}

#[derive(syn_derive::ToTokens)]
struct ToBytesFnItem {
    fn_item: FnItem
}

impl_from_ir!(ToBytesFnItem);

impl ToBytesFnItem {
    fn from_struct(fields: Vec<(syn::Ident, syn::Type)>) -> syn::Result<Self> {
        let ty_bytes_vec = utils::ty::bytes_vec();
        let ty_ret = utils::ty::bytes_result(&ty_bytes_vec);
        let ty_self = utils::ty::_self();

        let ident_result = utils::ident::result();
        let serialized_length_expr = utils::expr::serialized_length(&ty_self);

        let init_vec_stmt =
            utils::stmt::new_mut_vec_with_capacity(&ident_result, &serialized_length_expr);

        let serialize = fields.iter().map(|(i, _)| {
            let member = utils::member::_self(i);
            let expr_to_bytes = utils::expr::failable_to_bytes(&member);
            quote::quote!(#ident_result.extend(#expr_to_bytes);)
        }).collect::<Vec<_>>();

        let name = utils::ident::to_bytes();
        let ret_ty = utils::misc::ret_ty(&ty_ret);
        let block = parse_quote!({
            #init_vec_stmt
            #(#serialize)*
            Ok(#ident_result)
        });
        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, block).instanced()
        })
    }

    fn from_enum(variants: Vec<syn::Variant>) -> syn::Result<Self> {
        let ty_bytes_vec = utils::ty::bytes_vec();
        let ty_ret = utils::ty::bytes_result(&ty_bytes_vec);
        let name = utils::ident::to_bytes();
        let ret_ty = utils::misc::ret_ty(&ty_ret);
        let ident_result = utils::ident::result();

        let arms = variants.iter()
            .enumerate()
            .map(|(idx, v)| {
                let idx = idx as u8;
                let ident = &v.ident;
                let fields = variant_ident_vec(v);
                let left = match &v.fields {
                    syn::Fields::Unit => quote::quote!(Self::#ident),
                    syn::Fields::Named(_) => quote::quote!(Self::#ident { #(#fields),* }),
                    syn::Fields::Unnamed(_) => quote::quote!(Self::#ident( #(#fields),* ))
                };
                quote::quote!(#left => {
                    let mut #ident_result = odra::prelude::vec![#idx];
                    #(#ident_result.extend_from_slice(&#fields.to_bytes()?);)*
                    Ok(#ident_result)
                })
            })
            .collect::<Punctuated<_, Token![,]>>();
        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, match_self_expr(arms).as_block()).instanced()
        })
    }

    fn from_unit_enum(_variants: Vec<syn::Variant>) -> syn::Result<Self> {
        let ty_bytes_vec = utils::ty::bytes_vec();
        let ty_ret = utils::ty::bytes_result(&ty_bytes_vec);
        let name = utils::ident::to_bytes();

        let ret_ty = utils::misc::ret_ty(&ty_ret);
        let block = utils::expr::serialize_enum().as_block();

        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, block).instanced()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct SerializedLengthFnItem {
    fn_item: FnItem
}

impl_from_ir!(SerializedLengthFnItem);

impl SerializedLengthFnItem {
    fn from_struct(fields: Vec<(syn::Ident, syn::Type)>) -> syn::Result<Self> {
        let ty_usize = utils::ty::usize();
        let ident_result = utils::ident::result();

        let stmts = fields.iter().map(|(i, _)| {
            let member = utils::member::_self(i);
            let expr = utils::expr::serialized_length(&member);
            let stmt: syn::Stmt = parse_quote!(#ident_result += #expr;);
            stmt
        }).collect::<Vec<_>>();

        let name = utils::ident::serialized_length();
        let ret_ty = utils::misc::ret_ty(&ty_usize);
        let block = parse_quote!({
            let mut #ident_result = 0;
            #(#stmts)*
            #ident_result
        });
        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, block).instanced()
        })
    }

    fn from_enum(variants: Vec<syn::Variant>)  -> syn::Result<Self> {
        let ty_usize = utils::ty::usize();
        let name = utils::ident::serialized_length();
        let ret_ty = utils::misc::ret_ty(&ty_usize);
        let expr_u8_serialized_len = utils::expr::u8_serialized_len();
        
        let arms = variants.iter()
            .map(|v| {
                let ident = &v.ident;
                let fields = variant_ident_vec(v);
                let left = match &v.fields {
                    syn::Fields::Unit => quote::quote!(Self::#ident),
                    syn::Fields::Named(_) => quote::quote!(Self::#ident { #(#fields),* }),
                    syn::Fields::Unnamed(_) => quote::quote!(Self::#ident( #(#fields),* ))
                };
                quote::quote!(#left => #expr_u8_serialized_len #(+ #fields.serialized_length())* )
            })
            .collect::<Punctuated<_, Token![,]>>();
        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, match_self_expr(arms).as_block()).instanced()
        })
    }

    fn from_unit_enum(_variants: Vec<syn::Variant>) -> syn::Result<Self> {
        let ty_usize = utils::ty::usize();
        let name = utils::ident::serialized_length();
        let ret_ty = utils::misc::ret_ty(&ty_usize);
        let block = utils::expr::u8_serialized_len().as_block();
        Ok(Self {
            fn_item: FnItem::new(&name, vec![], ret_ty, block).instanced()
        })
    }
}

fn variant_ident_vec(variant: &syn::Variant) -> Vec<syn::Ident> {
    variant.fields
        .clone()
        .iter()
        .enumerate()
        .map(|(idx, i)| match &i.ident {
            Some(ident) => ident.clone(),
            None => format_ident!("f{}", idx)
        })
        .collect::<Vec<_>>()
}

fn match_self_expr<T: ToTokens>(arms: T) -> syn::Expr {
    parse_quote!(match self { #arms })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn test_struct() {
        let ir = test_utils::mock::custom_struct();
        let item = OdraTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[derive(Clone, PartialEq, Eq, Debug)]
            struct MyType {
                a: String,
                b: u32,
            }

            impl odra::casper_types::bytesrepr::FromBytes for MyType {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                    let (a, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                    let (b, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;

                    Ok((Self {
                        a,
                        b
                    }, bytes))
                }
            }

            impl odra::casper_types::bytesrepr::ToBytes for MyType {
                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                    let mut result = odra::prelude::vec::Vec::with_capacity(self.serialized_length());
                    result.extend(odra::casper_types::bytesrepr::ToBytes::to_bytes(&self.a)?);
                    result.extend(odra::casper_types::bytesrepr::ToBytes::to_bytes(&self.b)?);
                    Ok(result)
                }

                fn serialized_length(&self) -> usize {
                    let mut result = 0;
                    result += self.a.serialized_length();
                    result += self.b.serialized_length();
                    result
                }
            }

            impl odra::casper_types::CLTyped for MyType {
                fn cl_type() -> odra::casper_types::CLType {
                    odra::casper_types::CLType::Any
                }
            }

            impl odra::contract_def::HasEvents for MyType {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    odra::prelude::vec::Vec::new()
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    odra::prelude::BTreeMap::new()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_unit_enum() {
        let ir = test_utils::mock::custom_enum();
        let item = OdraTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[derive(Clone, PartialEq, Eq, Debug)]
            enum MyType {
                /// Description of A
                A = 10,
                /// Description of B
                B
            }

            impl odra::casper_types::bytesrepr::FromBytes for MyType {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                    let (result, bytes): (u8, _) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                    match result {
                        x if x == Self::A as u8 => Ok((Self::A, bytes)),
                        x if x == Self::B as u8 => Ok((Self::B, bytes)),
                        _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
                    }
                }
            }

            impl odra::casper_types::bytesrepr::ToBytes for MyType {
                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                    Ok(odra::prelude::vec![self.clone() as u8])
                }

                fn serialized_length(&self) -> usize {
                    odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH
                }
            }

            impl odra::casper_types::CLTyped for MyType {
                fn cl_type() -> odra::casper_types::CLType {
                    odra::casper_types::CLType::U8
                }
            }

            impl odra::contract_def::HasEvents for MyType {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    odra::prelude::vec::Vec::new()
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    odra::prelude::BTreeMap::new()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_complex_enum() {
        let ir = test_utils::mock::custom_complex_enum();
        let item = OdraTypeItem::try_from(&ir).unwrap();
        let expected = quote!(
            #[derive(Clone, PartialEq, Eq, Debug)]
            enum MyType {
                /// Description of A
                A { a: String, b: u32 },
                /// Description of B
                B(u32, String),
                /// Description of C
                C(),
                /// Description of D
                D {},
            }

            impl odra::casper_types::bytesrepr::FromBytes for MyType {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                    let (result, bytes): (u8, _) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                    match result {
                        0u8 => {
                            let (a, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                            let (b, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                            Ok((Self::A { a, b }, bytes))
                        },
                        1u8 => {
                            let (f0, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                            let (f1, bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                            Ok((Self::B(f0, f1), bytes))
                        },
                        2u8 => Ok((Self::C(), bytes)),
                        3u8 => Ok((Self::D {}, bytes)),
                        _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
                    }
                }
            }

            impl odra::casper_types::bytesrepr::ToBytes for MyType {
                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                    match self {
                        Self::A { a, b } => {
                            let mut result = odra::prelude::vec![0u8];
                            result.extend_from_slice(&a.to_bytes()?);
                            result.extend_from_slice(&b.to_bytes()?);
                            Ok(result)
                        },
                        Self::B(f0, f1) => {
                            let mut result = odra::prelude::vec![1u8];
                            result.extend_from_slice(&f0.to_bytes()?);
                            result.extend_from_slice(&f1.to_bytes()?);
                            Ok(result)
                        },
                        Self::C() => {
                            let mut result = odra::prelude::vec![2u8];
                            Ok(result)
                        },
                        Self::D {} => {
                            let mut result = odra::prelude::vec![3u8];
                            Ok(result)
                        }
                    }
                }

                fn serialized_length(&self) -> usize {
                    match self {
                        Self::A { a, b } => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH + a.serialized_length() + b.serialized_length(),
                        Self::B(f0, f1) => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH + f0.serialized_length() + f1.serialized_length(),
                        Self::C() => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH,
                        Self::D {} => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH,
                    }
                }
            }

            impl odra::casper_types::CLTyped for MyType {
                fn cl_type() -> odra::casper_types::CLType {
                    odra::casper_types::CLType::Any
                }
            }

            impl odra::contract_def::HasEvents for MyType {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    odra::prelude::vec::Vec::new()
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    odra::prelude::BTreeMap::new()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn test_union() {
        let ir = test_utils::mock::custom_union();
        let item = OdraTypeItem::try_from(&ir);
        assert!(item.is_err());
    }
}
