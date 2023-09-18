use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::{FnArg, PatType};

pub(crate) fn generate_fn_body<T>(
    args: T,
    entrypoint_name: &String,
    ret: &syn::ReturnType
) -> TokenStream
where
    T: IntoIterator<Item = syn::PatType>
{
    let args = parse_args(args);

    match ret {
        syn::ReturnType::Default => quote! {
            #args
            odra::call_contract::<()>(self.address, #entrypoint_name, &args, self.attached_value);
        },
        syn::ReturnType::Type(_, _) => quote! {
            #args
            odra::call_contract(self.address, #entrypoint_name, &args, self.attached_value)
        }
    }
}

pub(crate) fn filter_args<'a, T>(args: T) -> Vec<syn::PatType>
where
    T: IntoIterator<Item = &'a syn::FnArg>
{
    args.into_iter().filter_map(typed_arg).collect::<Vec<_>>()
}

pub(crate) fn build_ref(ref_ident: &Ident, struct_ident: &Ident) -> TokenStream {
    let ref_comment = format!("Reference to the [{}] contract instance.", struct_ident);
    quote! {
        #[derive(Clone)]
        #[doc = #ref_comment]
        pub struct #ref_ident {
            address: odra::types::Address,
            attached_value: Option<odra::types::Balance>,
        }

        impl #ref_ident {
            pub fn at(address: &odra::types::Address) -> Self {
                Self { address: *address, attached_value: None }
            }

            pub fn address(&self) -> &odra::types::Address {
                &self.address
            }

            pub fn with_tokens<T>(&self, amount: T) -> Self
            where T: Into<odra::types::Balance> {
                Self {
                    address: self.address,
                    attached_value: Some(amount.into()),
                }
            }
        }
    }
}

fn parse_args<T>(syn_args: T) -> TokenStream
where
    T: IntoIterator<Item = syn::PatType>
{
    let mut tokens = quote!(let mut args = odra::types::casper_types::RuntimeArgs::new(););
    tokens.append_all(syn_args.into_iter().map(|ty| {
        let pat = &*ty.pat;
        match *ty.ty {
            syn::Type::Reference(syn::TypeReference { ref elem, .. }) => match **elem {
                syn::Type::Slice { .. } => quote! {
                    #[cfg(any(feature = "casper", feature = "casper-livenet"))]
                    args.insert(stringify!(#pat), #pat.to_vec());
                    #[cfg(feature = "mock-vm")]
                    args.insert(stringify!(#pat), #pat.clone());
                },
                _ => quote!(args.insert(stringify!(#pat), #pat.clone());)
            },
            _ => quote!(args.insert(stringify!(#pat), #pat.clone());)
        }
    }));
    tokens.extend(quote!(args));

    quote!(let args = { #tokens };)
}

pub fn typed_arg(arg: &FnArg) -> Option<PatType> {
    match arg {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat) => Some(pat.clone())
    }
}

pub(crate) mod mock_vm {
    use proc_macro2::{Ident, TokenStream};
    use quote::quote;
    use syn::{punctuated::Punctuated, token::Comma, Variant};

    pub fn serialize_struct(struct_ident: &Ident, fields: &[Ident]) -> TokenStream {
        let fields_serialization = fields
            .iter()
            .map(|ident| quote!(odra::types::mock_vm::borsh::BorshSerialize::serialize(&self.#ident, writer)?;))
            .collect::<TokenStream>();

        let fields_deserialization = fields
            .iter()
            .map(|ident| quote!(#ident: odra::types::mock_vm::borsh::BorshDeserialize::deserialize(buf)?,))
            .collect::<TokenStream>();

        quote! {
            #[cfg(feature = "mock-vm")]
            impl odra::types::mock_vm::borsh::BorshSerialize for #struct_ident {
                fn serialize<W: odra::types::mock_vm::borsh::Write>(&self, writer: &mut W) -> odra::types::mock_vm::borsh::Result<()> {
                    odra::types::mock_vm::borsh::BorshSerialize::serialize(stringify!(#struct_ident), writer)?;

                    #fields_serialization
                    Ok(())
                }
            }

            #[cfg(feature = "mock-vm")]
            impl odra::types::mock_vm::borsh::BorshDeserialize for #struct_ident {

                fn deserialize(buf: &mut &[u8]) -> odra::types::Result<Self> {
                    let _ = <odra::prelude::string::String as odra::types::mock_vm::borsh::BorshDeserialize>::deserialize(buf)?;
                    Ok(Self {
                        #fields_deserialization
                    })
                }
            }

            #[cfg(feature = "mock-vm")]
            impl odra::types::Typed for #struct_ident {
                fn ty() -> odra::types::Type {
                    odra::types::Type::Any
                }
            }
        }
    }

    pub fn serialize_enum(struct_ident: &Ident, variants: &[Variant]) -> TokenStream {
        let variant_idx = variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                quote!(#struct_ident::#ident => #struct_ident::#ident as u8)
            })
            .collect::<Punctuated<TokenStream, Comma>>();

        let variant_idx_serialization = quote! {
            let variant_idx: u8 = match self {
                #variant_idx
            };
            writer.write_all(&variant_idx.to_le_bytes())?;
        };

        let fields_deserialization = variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                quote!(x if x == #struct_ident::#ident as u8 => #struct_ident::#ident)
            })
            .collect::<Punctuated<TokenStream, Comma>>();

        quote! {
            #[cfg(feature = "mock-vm")]
            impl odra::types::mock_vm::borsh::BorshSerialize for #struct_ident {
                fn serialize<W: odra::types::Write>(&self, writer: &mut W) -> odra::types::Result<()> {
                    #variant_idx_serialization
                    Ok(())
                }
            }

            #[cfg(feature = "mock-vm")]
            impl odra::types::mock_vm::borsh::BorshDeserialize for #struct_ident {
                fn deserialize(buf: &mut &[u8]) -> odra::types::Result<Self> {
                    let variant_idx: u8 = odra::types::mock_vm::borsh::BorshDeserialize::deserialize(buf)?;
                    let return_value = match variant_idx {
                        #fields_deserialization,
                        _ => return Err(
                            odra::types::Error::new(
                                odra::types::ErrorKind::InvalidInput, "Unexpected variant index"
                        ))
                    };
                    Ok(return_value)
                }
            }

            #[cfg(feature = "mock-vm")]
            impl odra::types::Typed for #struct_ident {
                fn ty() -> odra::types::Type {
                    odra::types::Type::U8
                }
            }
        }
    }
}

pub(crate) mod casper {

    use proc_macro2::{Ident, TokenStream};
    use quote::{format_ident, quote, TokenStreamExt};
    use syn::{punctuated::Punctuated, token::Comma, Variant};

    pub fn serialize_struct(prefix: &str, struct_ident: &Ident, fields: &[Ident]) -> TokenStream {
        let name_literal = format_ident!("{prefix}{struct_ident}");
        let name_literal = quote!(stringify!(#name_literal));

        let deserialize_fields = fields
            .iter()
            .map(|ident| quote!(let (#ident, bytes) = odra::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;))
            .collect::<TokenStream>();

        let construct_struct = fields
            .iter()
            .map(|ident| quote! { #ident, })
            .collect::<TokenStream>();

        let mut sum_serialized_lengths = quote! {
            size += #name_literal.serialized_length();
        };
        sum_serialized_lengths.append_all(
            fields
                .iter()
                .map(|ident| quote!(size += self.#ident.serialized_length();))
        );

        let append_bytes = fields
            .iter()
            .flat_map(|ident| {
                quote! {
                    vec.extend(&self.#ident.to_bytes()?);
                }
            })
            .collect::<TokenStream>();

        quote! {
            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::bytesrepr::FromBytes for #struct_ident {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper::casper_types::bytesrepr::Error> {
                    let (_, bytes): (odra::prelude::string::String, _) = odra::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                    #deserialize_fields
                    let value = #struct_ident {
                        #construct_struct
                    };
                    Ok((value, bytes))
                }
            }

            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::bytesrepr::ToBytes for #struct_ident {
                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper::casper_types::bytesrepr::Error> {
                    let mut vec = odra::prelude::vec::Vec::with_capacity(self.serialized_length());
                    vec.append(&mut #name_literal.to_bytes()?);
                    #append_bytes
                    Ok(vec)
                }

                fn serialized_length(&self) -> usize {
                    let mut size = 0;
                    #sum_serialized_lengths
                    size
                }
            }

            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::CLTyped for #struct_ident {
                fn cl_type() -> odra::casper::casper_types::CLType {
                    odra::casper::casper_types::CLType::Any
                }
            }
        }
    }

    pub fn serialize_enum(enum_ident: &Ident, variants: &[Variant]) -> TokenStream {
        let from_bytes_code = enum_from_bytes(enum_ident, variants);

        quote! {
            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::bytesrepr::FromBytes for #enum_ident {
                #from_bytes_code
            }

            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::bytesrepr::ToBytes for #enum_ident {
                fn serialized_length(&self) -> usize {
                    (self.clone() as u32).serialized_length()
                }

                fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper::casper_types::bytesrepr::Error> {
                    (self.clone() as u32).to_bytes()
                }
            }

            #[cfg(any(feature = "casper", feature = "casper-livenet"))]
            impl odra::casper::casper_types::CLTyped for #enum_ident {
                fn cl_type() -> odra::casper::casper_types::CLType {
                    odra::casper::casper_types::CLType::U32
                }
            }
        }
    }

    fn enum_from_bytes(enum_ident: &Ident, variants: &[Variant]) -> TokenStream {
        let append_bytes = variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                quote! {
                    x if x == #enum_ident::#ident as u32 => core::result::Result::Ok((#enum_ident::#ident, bytes))
                }
            })
            .collect::<Punctuated<TokenStream, Comma>>();

        quote! {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper::casper_types::bytesrepr::Error> {
                let (variant, bytes): (u32, _) = odra::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                match variant {
                    #append_bytes,
                    _ => core::result::Result::Err(odra::casper::casper_types::bytesrepr::Error::Formatting),
                }
            }
        }
    }
}
