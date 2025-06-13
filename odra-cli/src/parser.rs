use clap::{builder::TypedValueParser, error::ErrorKind, Arg, Command, Error};
use odra::{
    casper_types::CLValue,
    schema::{casper_contract_schema::NamedCLType, NamedCLTyped}
};

use crate::types;

#[derive(Clone)]
#[non_exhaustive]
pub struct GenericCLValueParser<E: NamedCLTyped + Clone + Send + Sync + 'static> {
    _marker: std::marker::PhantomData<E>
}

impl<E: NamedCLTyped + Clone + Send + Sync + 'static> GenericCLValueParser<E> {
    /// Parse non-empty string values
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData
        }
    }
}

impl<E: NamedCLTyped + Clone + Send + Sync + 'static> TypedValueParser for GenericCLValueParser<E> {
    type Value = CLValue;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr
    ) -> Result<Self::Value, Error> {
        let value = value
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))?;
        let ty = E::ty();
        let bytes = types::into_bytes(&ty, value).map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "unknown argument".to_string());
            let message = format!(
                "Failed to parse arg {} with value '{}' for type '{:?}':\nCaused by: {}\n",
                arg, value, ty, err
            );
            Error::raw(ErrorKind::InvalidValue, message).with_cmd(cmd)
        })?;
        let cl_type = types::named_cl_type_to_cl_type(&ty);
        Ok(CLValue::from_components(cl_type, bytes))
    }
}

impl<E: NamedCLTyped + Clone + Send + Sync + 'static> Default for GenericCLValueParser<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct CLTypedParser {
    ty: NamedCLType
}

impl CLTypedParser {
    /// Parse non-empty string values
    pub fn new(ty: NamedCLType) -> Self {
        Self { ty }
    }
}

impl TypedValueParser for CLTypedParser {
    type Value = CLValue;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr
    ) -> Result<Self::Value, Error> {
        let value = value
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))?;
        let bytes = types::into_bytes(&self.ty, value).map_err(|err| {
            let arg = arg
                .map(|a| a.to_string())
                .unwrap_or_else(|| "unknown argument".to_string());
            let message = format!(
                "Failed to parse arg {} with value '{}' for type '{:?}':\nCaused by: {}\n",
                arg, value, &self.ty, err
            );
            Error::raw(ErrorKind::InvalidValue, message).with_cmd(cmd)
        })?;
        let cl_type = types::named_cl_type_to_cl_type(&self.ty);
        Ok(CLValue::from_components(cl_type, bytes))
    }
}
