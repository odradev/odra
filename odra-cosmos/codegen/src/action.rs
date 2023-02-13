use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

pub fn ident() -> Ident {
    format_ident!("action")
}

pub fn action_ty_ident() -> Ident {
    format_ident!("Action")
}

pub fn struct_code() -> TokenStream {
    quote::quote! {
        #[derive(Debug, PartialEq)]
        struct Action {
            pub name: String,
            pub args: Vec<Vec<u8>>,
        }
    }
}

pub fn deserialization_code() -> TokenStream {
    quote::quote! {
        impl odra::cosmos::serde::Serialize for Action {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: odra::cosmos::serde::Serializer {

                let mut s = serializer.serialize_struct("Action", 2)?;
                odra::cosmos::serde::ser::SerializeStruct::serialize_field(&mut s, "name", &self.name)?;
                odra::cosmos::serde::ser::SerializeStruct::serialize_field(&mut s, "args", &self.args)?;
                odra::cosmos::serde::ser::SerializeStruct::end(s)
            }
        }
        struct ActionVisitor;

        impl<'de> odra::cosmos::serde::de::Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an Action")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: odra::cosmos::serde::de::MapAccess<'de>, {
                    let mut name: Option<String> = None;
                    let mut args: Option<Vec<Vec<u8>>> = None;
                    while let Some(key) =
                        match odra::cosmos::serde::de::MapAccess::next_key::<String>(&mut map) {
                            Ok(val) => val,
                            Err(err) => return Err(err)
                        }
                    {
                        match key.as_str() {
                            "name" => {
                                name = Some(
                                    match odra::cosmos::serde::de::MapAccess::next_value::<String>(&mut map) {
                                        Ok(val) => val,
                                        Err(err) => return Err(err)
                                    },
                                );
                            },
                            "args" => {
                                args = Some(
                                    match odra::cosmos::serde::de::MapAccess::next_value::<Vec<Vec<u8>>>(&mut map) {
                                        Ok(val) => val,
                                        Err(err) => return Err(err)
                                    },
                                );
                            },
                            _ => odra::contract_env::revert(Error::UnknownAction),
                        }
                    }
                    Ok(Action { name: name.unwrap(), args: args.unwrap() })
            }
        }

        impl <'de> odra::cosmos::serde::Deserialize<'de> for Action {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: odra::cosmos::serde::Deserializer<'de> {
                    deserializer.deserialize_struct(
                        "Action",
                        &["name", "args"],
                        ActionVisitor
                    )
            }
        }

        fn get_arg<T: odra::types::OdraType>(bytes: Vec<u8>) -> T {
            T::deser(bytes).unwrap()
        }

        odra::execution_error! {
            enum Error {
                UnknownAction => 1000
            }
        }
    }
}
