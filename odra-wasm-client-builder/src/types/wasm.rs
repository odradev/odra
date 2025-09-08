use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{NamedCLType, StructMember, Type};
use quote::{format_ident, ToTokens};
use syn::parse_quote;

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
    List(Box<WasmType>),
    Result {
        ok: Box<WasmType>,
        err: Box<WasmType>
    }
}

impl WasmType {
    pub fn field(&self, member: &StructMember) -> syn::Field {
        let field_name = format_ident!("{}", member.name);
        let js_name = member.name.to_case(Case::Camel);
        parse_quote!(#[wasm_bindgen(js_name = #js_name)] #field_name: #self)
    }

    pub fn is_wrapped_type(&self) -> bool {
        match self {
            WasmType::Result { ok, err } if ok.is_wrapped_type() && err.is_wrapped_type() => true,
            WasmType::U128
            | WasmType::U256
            | WasmType::U512
            | WasmType::Address
            | WasmType::URef
            | WasmType::PublicKey => true,
            _ => false
        }
    }

    pub fn getter_code(&self, member: &StructMember) -> proc_macro2::TokenStream {
        let ident = format_ident!("{}", member.name);

        match self {
            t if t.is_wrapped_type() => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    self.#ident.clone().into()
                }
            },
            WasmType::List(_) => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    self.#ident.into_iter().map(Into::into).collect()
                }
            },
            WasmType::JsValueList => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    self.#ident.iter().map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null())).collect()
                }
            },
            WasmType::JsValue => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    JsValue::from_serde(v).unwrap_or(JsValue::null())
                }
            },
            WasmType::Option(box WasmType::JsValue) => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    self.#ident.map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null()))
                }
            },
            WasmType::Option(box WasmType::JsValueList) => quote::quote! {
                #[wasm_bindgen(getter)]
                pub fn #ident(&self) -> #self {
                    self.#ident.iter().map(|v| JsValue::from_serde(v).unwrap_or(JsValue::null())).collect()
                }
            },
            WasmType::Option(e) if e.is_wrapped_type() => {
                quote::quote! {
                    #[wasm_bindgen(getter)]
                    pub fn #ident(&self) -> #self {
                        self.#ident.map(|v| v.into())
                    }
                }
            }
            WasmType::Result { ok: _, err: _ } => quote::quote! {
                panic!("Unsupported type for getter");
            },
            _ => quote::quote! {
                panic!("Unsupported type for getter");
            }
        }
    }

    pub fn setter_code(&self, member: &StructMember) -> proc_macro2::TokenStream {
        let ident = format_ident!("set_{}", member.name);
        let field_name = format_ident!("{}", member.name);
        let ty = self.to_token_stream();

        match self {
            t if t.is_wrapped_type() => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into();
                }
            },
            WasmType::List(_) => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_iter().map(Into::into).collect();
                }
            },
            WasmType::JsValueList => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_iter().filter_map(|js_value| {
                        js_value.into_serde().ok()
                    }).collect();
                }
            },
            WasmType::JsValue => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.into_serde().expect("Failed to deserialize JS value");
                }
            },
            WasmType::Option(box WasmType::JsValue) => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.map(|v| v.into_serde().ok()).flatten();
                }
            },
            WasmType::Option(box WasmType::JsValueList) => quote::quote! {
                #[wasm_bindgen(setter)]
                pub fn #ident(&mut self, value: #ty) {
                    self.#field_name = value.map(|v| v.into_iter().filter_map(|js_value| {
                        js_value.into_serde().ok()
                    }).collect());
                }
            },
            WasmType::Option(e) if e.is_wrapped_type() => {
                quote::quote! {
                    #[wasm_bindgen(setter)]
                    pub fn #ident(&mut self, value: #ty) {
                        self.#field_name = value.map(|v| v.into());
                    }
                }
            }
            WasmType::Result { ok: _, err: _ } => quote::quote! {
                panic!("Unsupported type for setter");
            },
            _ => quote::quote! {
                panic!("Unsupported type for setter");
            }
        }
    }
}

impl ToTokens for WasmType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = match self {
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
            WasmType::Option(inner) => {
                // let inner = named_cl_type_to_wasm_type(&Type(*inner.clone()));
                quote::quote!(Option<#inner>)
            }
            WasmType::Custom(name) => {
                let ident = format_ident!("{}", name);
                quote::quote!(#ident)
            }
            WasmType::Result { ok, err } => {
                quote::quote!(Result<#ok, #err>)
            }
            WasmType::List(inner) => {
                quote::quote!(Vec<#inner>)
            }
        };
        tokens.extend(ty);
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
        NamedCLType::List(named_cltype) => {
            let t = *named_cltype.to_owned();
            match t {
                NamedCLType::Option(_named_cltype) => WasmType::JsValueList,
                NamedCLType::List(_named_cltype) => WasmType::JsValueList,
                NamedCLType::ByteArray(_) => WasmType::JsValueList,
                NamedCLType::Result { ok: _, err: _ } => WasmType::JsValueList,
                NamedCLType::Map { key: _, value: _ } => WasmType::JsValueList,
                NamedCLType::Tuple1(_) => WasmType::JsValueList,
                NamedCLType::Tuple2(_) => WasmType::JsValueList,
                NamedCLType::Tuple3(_) => WasmType::JsValueList,
                // NamedCLType::Custom(_) => WasmType::JsValueList,
                _ => {
                    let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
                    WasmType::List(Box::new(inner))
                }
            }
        }
        NamedCLType::ByteArray(_n) => WasmType::Bytes,
        NamedCLType::Result { ok, err } => {
            let ok = named_cl_type_to_wasm_type(&Type(*ok.clone()));
            let err = named_cl_type_to_wasm_type(&Type(*err.clone()));
            WasmType::Result {
                ok: Box::new(ok),
                err: Box::new(err)
            }
        }
        NamedCLType::Map { key: _, value: _ } => WasmType::JsValue,
        NamedCLType::Tuple1(_) => WasmType::JsValue,
        NamedCLType::Tuple2(_) => WasmType::JsValue,
        NamedCLType::Tuple3(_) => WasmType::JsValue,
        NamedCLType::Custom(name) => WasmType::Custom(name.to_string())
    }
}
