use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{Argument, NamedCLType, StructMember, Type};
use quote::{format_ident, ToTokens};
use syn::parse_quote;

use crate::types::OdraType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WasmType {
    Bool,
    I32,
    I64,
    U8,
    U32,
    U64,
    U128,
    U256,
    U512,
    Unit,
    String,
    URef,
    PublicKey,
    JsValue,
    JsValueList,
    Bytes,
    Address,
    Custom(String),
    Option(Box<WasmType>),
    List(Box<WasmType>)
}

impl WasmType {
    pub fn field_def(member: &StructMember) -> syn::Field {
        let ty = Self::from(&member.ty);
        let field_name = format_ident!("{}", member.name);
        let js_name = member.name.to_case(Case::Camel);
        parse_quote!(#[wasm_bindgen(js_name = #js_name)] #field_name: #ty)
    }

    pub fn field_init(member: &StructMember) -> proc_macro2::TokenStream {
        let field_name = format_ident!("{}", member.name);
        let odra_ty = OdraType::from(&member.ty);
        let wasm_ty = WasmType::from(&member.ty);
        // If the WASM type is equal to the Odra type, we can use the field directly
        if wasm_ty == odra_ty {
            return parse_quote!(#field_name);
        }

        if matches!(wasm_ty, WasmType::Option(_)) {
            parse_quote!(#field_name: #field_name.map(Into::into))
        } else if matches!(wasm_ty, WasmType::List(_)) {
            parse_quote!(#field_name: #field_name.into_iter().map(Into::into).collect())
        } else if matches!(wasm_ty, WasmType::JsValueList) {
            parse_quote!(#field_name: #field_name.into_iter().map(|v| v.into_serde().expect("Failed to deserialize JsValue")).collect())
        } else if matches!(wasm_ty, WasmType::JsValue) {
            parse_quote!(#field_name: #field_name.into_serde().expect("Failed to deserialize JsValue"))
        } else {
            parse_quote!(#field_name: #field_name.into())
        }
    }

    pub fn getter_code(member: &StructMember) -> Option<proc_macro2::TokenStream> {
        let ty = Self::from(&member.ty);
        let ident = format_ident!("{}", member.name);

        match &ty {
            t if t.is_wrapped_type() => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.clone().into()
                }
            }),
            WasmType::List(t) if !t.is_primitive() => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.into_iter().map(Into::into).collect()
                }
            }),
            WasmType::JsValueList => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.iter().map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null())).collect()
                }
            }),
            WasmType::JsValue => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    JsValue::from_serde(self.#ident).unwrap_or(JsValue::null())
                }
            }),
            WasmType::Option(box WasmType::JsValue) => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null()))
                }
            }),
            WasmType::Option(box WasmType::JsValueList) => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.iter().map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null())).collect()
                }
            }),
            WasmType::Option(e) if e.is_wrapped_type() => Some(quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #ty {
                    self.#ident.map(Into::into)
                }
            }),
            _ => None
        }
    }

    pub fn setter_code(member: &StructMember) -> Option<proc_macro2::TokenStream> {
        let ty = Self::from(&member.ty);
        let ident = format_ident!("set_{}", member.name);
        let field_name = format_ident!("{}", member.name);

        match &ty {
            t if t.is_wrapped_type() => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into();
                }
            }),
            WasmType::List(t) if !t.is_primitive() => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_iter().map(Into::into).collect();
                }
            }),
            WasmType::JsValueList => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_iter().filter_map(|js_value| {
                        js_value.into_serde().ok()
                    }).collect();
                }
            }),
            WasmType::JsValue => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_serde().expect("Failed to deserialize JS value");
                }
            }),
            WasmType::Option(box WasmType::JsValue) => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.map(|v| v.into_serde().ok()).flatten();
                }
            }),
            WasmType::Option(box WasmType::JsValueList) => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.map(|v| v.into_iter().filter_map(|js_value| {
                        js_value.into_serde().ok()
                    }).collect());
                }
            }),
            WasmType::Option(e) if e.is_wrapped_type() => Some(quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.map(Into::into);
                }
            }),
            _ => None
        }
    }

    pub fn runtime_arg(arg: &Argument) -> proc_macro2::TokenStream {
        let wasm_ty = WasmType::from(&arg.ty);
        let odra_ty = OdraType::from(&arg.ty);
        let arg_name = format_ident!("{}", arg.name);
        let arg_str = &arg.name;
        if wasm_ty.is_wrapped_type() {
            return parse_quote!(#arg_str => (*#arg_name).clone());
        } else if matches!(wasm_ty, WasmType::Option(ref e) if e.is_wrapped_type()) {
            if let OdraType::Option(inner_odra_ty) = odra_ty {
                return parse_quote!(#arg_str => #arg_name.map(Into::<#inner_odra_ty>::into));
            }
            unreachable!("Expected OdraType::Option");
        } else if matches!(wasm_ty, WasmType::Option(box WasmType::Bytes)) {
            return parse_quote!(#arg_str => #arg_name);
        } else if matches!(wasm_ty, WasmType::List(e) if e.is_wrapped_type()) {
            return parse_quote!(#arg_str => #arg_name.into_iter().map(Into::into).collect::<#odra_ty>());
        } else {
            return parse_quote!(#arg_str => #arg_name);
        }
    }

    pub fn parse_js_value_arg(arg: &Argument) -> Option<syn::Stmt> {
        let arg_name = format_ident!("{}", arg.name);
        let odra_type = OdraType::from(&arg.ty);
        let wasm_type = Self::from(&arg.ty);
        match wasm_type {
            WasmType::JsValue | WasmType::JsValueList => {
                if matches!(arg.ty.0, NamedCLType::List(_))
                    || matches!(arg.ty.0, NamedCLType::ByteArray(_))
                {
                    Some(parse_quote! {
                        let #arg_name: #odra_type = #arg_name
                            .into_iter()
                            .map(|js_value| {
                                js_value
                                    .into_serde()
                                    .map_err(|err| JsError::new(&format!("{:?}", err)))
                            })
                            .collect::<Result<_, _>>()?;
                    })
                } else {
                    Some(
                        parse_quote!(#arg_name.into_serde::<#odra_type>().map_err(|err| JsError::new(&format!("{:?}", err)))?)
                    )
                }
            }
            _ => None
        }
    }

    pub fn parse_entry_point_arg(fn_arg: &Argument) -> syn::FnArg {
        let arg_ident = format_ident!("{}", fn_arg.name);
        let js_name = fn_arg.name.to_case(Case::Camel);
        let ty = Self::from(&fn_arg.ty);
        parse_quote!(#[wasm_bindgen(js_name = #js_name)] #arg_ident: #ty)
    }

    pub fn parse_return_expr(return_ty: &Type) -> syn::Expr {
        match Self::from(return_ty) {
            x if x.is_wrapped_type() => parse_quote!(Ok(result.0.into())),
            WasmType::JsValueList => parse_quote! {
                result.0.into_iter()
                    .map(|v| JsValue::from_serde(&v))
                    .collect::<Result<Vec<JsValue>, _>>()
                    .map_err(|err| JsError::new(&format!("{:?}", err)))
            },
            WasmType::JsValue => parse_quote! {
                JsValue::from_serde(&result.0).map_err(|err| JsError::new(&format!("{:?}", err)))
            },
            WasmType::Option(inner) if inner.is_wrapped_type() => {
                parse_quote!(Ok(result.0.map(Into::into)))
            }
            WasmType::Option(_) => parse_quote!(Ok(result.0)),
            WasmType::Bytes => parse_quote!(Ok(result.0.to_vec())),
            WasmType::Custom(_) => parse_quote!(Ok(result.0)),
            _ => parse_quote!(Ok(result.0))
        }
    }

    fn is_wrapped_type(&self) -> bool {
        match self {
            WasmType::U128
            | WasmType::U256
            | WasmType::U512
            | WasmType::Address
            | WasmType::URef
            | WasmType::PublicKey => true,
            _ => false
        }
    }

    fn is_primitive(&self) -> bool {
        match self {
            WasmType::Bool
            | WasmType::I32
            | WasmType::I64
            | WasmType::U8
            | WasmType::U32
            | WasmType::U64 => true,
            _ => false
        }
    }
}

impl ToTokens for WasmType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            WasmType::Bool => quote::quote!(bool),
            WasmType::I32 => quote::quote!(i32),
            WasmType::I64 => quote::quote!(i64),
            WasmType::U8 => quote::quote!(u8),
            WasmType::U32 => quote::quote!(u32),
            WasmType::U64 => quote::quote!(u64),
            WasmType::U128 => quote::quote!(odra_wasm_client::types::U128),
            WasmType::U256 => quote::quote!(odra_wasm_client::types::U256),
            WasmType::U512 => quote::quote!(odra_wasm_client::types::U512),
            WasmType::Address => quote::quote!(odra_wasm_client::types::Address),
            WasmType::URef => quote::quote!(odra_wasm_client::types::URef),
            WasmType::PublicKey => quote::quote!(odra_wasm_client::types::PublicKey),
            WasmType::Unit => quote::quote!(()),
            WasmType::String => quote::quote!(String),
            WasmType::JsValue => quote::quote!(JsValue),
            WasmType::JsValueList => quote::quote!(Vec<JsValue>),
            WasmType::Bytes => quote::quote!(Vec<u8>),
            WasmType::Option(inner) => quote::quote!(Option<#inner>),
            WasmType::Custom(name) => {
                let ident = format_ident!("{}", name);
                quote::quote!(#ident)
            }
            WasmType::List(inner) => quote::quote!(Vec<#inner>)
        });
    }
}

impl From<&Type> for WasmType {
    fn from(ty: &Type) -> Self {
        named_cl_type_to_wasm_type(ty)
    }
}

fn named_cl_type_to_wasm_type(ty: &Type) -> WasmType {
    match &ty.0 {
        NamedCLType::Bool => WasmType::Bool,
        NamedCLType::I32 => WasmType::I32,
        NamedCLType::I64 => WasmType::I64,
        NamedCLType::U8 => WasmType::U8,
        NamedCLType::U32 => WasmType::U32,
        NamedCLType::U64 => WasmType::U64,
        NamedCLType::U128 => WasmType::U128,
        NamedCLType::U256 => WasmType::U256,
        NamedCLType::U512 => WasmType::U512,
        NamedCLType::Unit => WasmType::Unit,
        NamedCLType::String => WasmType::String,
        NamedCLType::Key => WasmType::Address,
        NamedCLType::URef => WasmType::URef,
        NamedCLType::PublicKey => WasmType::PublicKey,
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
            WasmType::Option(Box::new(inner))
        }
        NamedCLType::List(named_cltype) => match *named_cltype.to_owned() {
            // List of complex types are not supported so we treat them as JsValueList
            NamedCLType::Option(_)
            | NamedCLType::List(_)
            | NamedCLType::ByteArray(_)
            | NamedCLType::Result { ok: _, err: _ }
            | NamedCLType::Map { key: _, value: _ }
            | NamedCLType::Tuple1(_)
            | NamedCLType::Tuple2(_)
            | NamedCLType::Tuple3(_) => WasmType::JsValueList,
            _ => {
                let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
                WasmType::List(Box::new(inner))
            }
        },
        NamedCLType::ByteArray(_) => WasmType::Bytes,
        NamedCLType::Result { ok: _, err: _ } => WasmType::JsValue,
        NamedCLType::Map { key: _, value: _ }
        | NamedCLType::Tuple1(_)
        | NamedCLType::Tuple2(_)
        | NamedCLType::Tuple3(_) => WasmType::JsValue,
        NamedCLType::Custom(name) => WasmType::Custom(name.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn test_wasm_type_conversion() {
        let wasm_type = WasmType::from(&Type(NamedCLType::I32));
        let expected = WasmType::I32;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::I64));
        let expected = WasmType::I64;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U8));
        let expected = WasmType::U8;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U32));
        let expected = WasmType::U32;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U64));
        let expected = WasmType::U64;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U128));
        let expected = WasmType::U128;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U256));
        let expected = WasmType::U256;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U512));
        let expected = WasmType::U512;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Unit));
        let expected = WasmType::Unit;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::String));
        let expected = WasmType::String;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::URef));
        let expected = WasmType::URef;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::PublicKey));
        let expected = WasmType::PublicKey;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Key));
        let expected = WasmType::Address;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::ByteArray(10)));
        let expected = WasmType::Bytes;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Custom("MyType".to_string())));
        let expected = WasmType::Custom("MyType".to_string());
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Option(Box::new(NamedCLType::U128))));
        let expected = WasmType::Option(Box::new(WasmType::U128));
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Option(Box::new(NamedCLType::Option(
            Box::new(NamedCLType::U128)
        )))));
        let expected = WasmType::Option(Box::new(WasmType::Option(Box::new(WasmType::U128))));
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple1([Box::new(NamedCLType::U128)]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple2([
            Box::new(NamedCLType::U128),
            Box::new(NamedCLType::U256)
        ]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple3([
            Box::new(NamedCLType::U128),
            Box::new(NamedCLType::U256),
            Box::new(NamedCLType::U512)
        ]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Map {
            key: Box::new(NamedCLType::U128),
            value: Box::new(NamedCLType::String)
        });
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Result {
            ok: Box::new(NamedCLType::U128),
            err: Box::new(NamedCLType::String)
        });
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::Custom(
            "MyType".to_string()
        ))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::List(Box::new(WasmType::Custom("MyType".to_string())));
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::Option(Box::new(
            NamedCLType::U128
        )))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValueList;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::ByteArray(32))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValueList;
        assert_eq!(wasm_type, expected);
    }

    #[test]
    fn test_field_def() {
        let field = StructMember {
            name: "test".to_string(),
            description: None,
            ty: Type(NamedCLType::U128)
        };
        let tokens = WasmType::field_def(&field);
        let expected =
            parse_quote!(#[wasm_bindgen(js_name = "test")] test: odra_wasm_client::types::U128);
        assert_eq!(tokens, expected);

        let field = StructMember {
            name: "test_field_rust_style".to_string(),
            description: None,
            ty: Type(NamedCLType::String)
        };
        let tokens = WasmType::field_def(&field);
        let expected = parse_quote!(#[wasm_bindgen(js_name = "testFieldRustStyle")] test_field_rust_style: String);
        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_init_string_field() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::String)
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field);
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_big_int_field() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::U128)
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field: test_field.into());
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_map_field() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Map {
                key: Box::new(NamedCLType::String),
                value: Box::new(NamedCLType::U128)
            })
        };
        let tokens = WasmType::field_init(&field);
        let expected =
            quote!(test_field: test_field.into_serde().expect("Failed to deserialize JsValue"));
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_vec_custom_type_field() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::Custom(
                "MyType".to_string()
            ))))
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field);
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_vec_vec_custom_type_field() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::List(Box::new(
                NamedCLType::Custom("MyType".to_string())
            )))))
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field: test_field.into_iter().map(|v| v.into_serde().expect("Failed to deserialize JsValue")).collect());
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_result() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Result {
                ok: Box::new(NamedCLType::Custom("MyType".to_string())),
                err: Box::new(NamedCLType::String)
            })
        };
        let tokens = WasmType::field_init(&field);
        let expected =
            quote!(test_field: test_field.into_serde().expect("Failed to deserialize JsValue"));
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_vec_address() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::Key)))
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field: test_field.into_iter().map(Into::into).collect());
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_option_address() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Option(Box::new(NamedCLType::Key)))
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field: test_field.map(Into::into));
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_init_option_string() {
        let field = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Option(Box::new(NamedCLType::String)))
        };
        let tokens = WasmType::field_init(&field);
        let expected = quote!(test_field);
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_primitive_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::U32)
        };

        let actual = WasmType::getter_code(&member);
        assert!(actual.is_none());
    }

    #[test]
    fn test_address_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Key)
        };

        let actual = WasmType::getter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(getter)]
            pub fn test_field(&self) -> odra_wasm_client::types::Address {
                self.test_field.clone().into()
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_address_option_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Option(Box::new(NamedCLType::Key)))
        };

        let actual = WasmType::getter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(getter)]
            pub fn test_field(&self) -> Option<odra_wasm_client::types::Address> {
                self.test_field.map(Into::into)
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_map_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Map {
                key: Box::new(NamedCLType::U128),
                value: Box::new(NamedCLType::String)
            })
        };

        let actual = WasmType::getter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(getter)]
            pub fn test_field(&self) -> JsValue {
                JsValue::from_serde(self.test_field).unwrap_or(JsValue::null())
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_vec_u32_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::U32)))
        };

        let actual = WasmType::getter_code(&member);
        assert!(actual.is_none());
    }

    #[test]
    fn test_vec_u256_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::U256)))
        };

        let actual = WasmType::getter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(getter)]
            pub fn test_field(&self) -> Vec<odra_wasm_client::types::U256> {
                self.test_field.into_iter().map(Into::into).collect()
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_vec_map_getter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::Map {
                key: Box::new(NamedCLType::U256),
                value: Box::new(NamedCLType::String)
            })))
        };

        let actual = WasmType::getter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(getter)]
            pub fn test_field(&self) -> Vec<JsValue> {
                self.test_field.iter().map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null())).collect()
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_primitive_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::U32)
        };

        let actual = WasmType::setter_code(&member);
        assert!(actual.is_none());
    }

    #[test]
    fn test_address_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Key)
        };

        let actual = WasmType::setter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(setter)]
            pub fn set_test_field(&mut self, value: odra_wasm_client::types::Address) {
                self.test_field = value.into();
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_address_option_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Option(Box::new(NamedCLType::Key)))
        };

        let actual = WasmType::setter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(setter)]
            pub fn set_test_field(&mut self, value: Option<odra_wasm_client::types::Address>) {
                self.test_field = value.map(Into::into);
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_map_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::Map {
                key: Box::new(NamedCLType::U128),
                value: Box::new(NamedCLType::String)
            })
        };

        let actual = WasmType::setter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(setter)]
            pub fn set_test_field(&mut self, value: JsValue) {
                self.test_field = value.into_serde().expect("Failed to deserialize JS value");
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_vec_u32_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::U32)))
        };

        let actual = WasmType::setter_code(&member);
        assert!(actual.is_none());
    }

    #[test]
    fn test_vec_u256_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::U256)))
        };

        let actual = WasmType::setter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(setter)]
            pub fn set_test_field(&mut self, value: Vec<odra_wasm_client::types::U256>) {
                self.test_field = value.into_iter().map(Into::into).collect();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_vec_map_setter() {
        let member = StructMember {
            name: "test_field".to_string(),
            description: None,
            ty: Type(NamedCLType::List(Box::new(NamedCLType::Map {
                key: Box::new(NamedCLType::U256),
                value: Box::new(NamedCLType::String)
            })))
        };

        let actual = WasmType::setter_code(&member).unwrap();
        let expected = quote! {
            #[wasm_bindgen(setter)]
            pub fn set_test_field(&mut self, value: Vec<JsValue>) {
                self.test_field = value.into_iter().filter_map(|js_value| { js_value.into_serde().ok() }).collect();
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
