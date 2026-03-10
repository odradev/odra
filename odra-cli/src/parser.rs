use clap::{builder::TypedValueParser, error::ErrorKind, Arg, Command, Error};
use odra::{
    casper_types::{CLType, CLValue, U512},
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
        let bytes = types::into_bytes(&ty, value)
            .map_err(|err| make_parse_error(cmd, arg, value, &ty, err))?;
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
        let bytes = types::into_bytes(&self.ty, value)
            .map_err(|err| make_parse_error(cmd, arg, value, &self.ty, err))?;
        let cl_type = types::named_cl_type_to_cl_type(&self.ty);
        Ok(CLValue::from_components(cl_type, bytes))
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct CsprTokenAmountParser;

impl TypedValueParser for CsprTokenAmountParser {
    type Value = U512;

    fn parse_ref(
        &self,
        cmd: &Command,
        _arg: Option<&Arg>,
        value: &std::ffi::OsStr
    ) -> Result<Self::Value, Error> {
        let value = value
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))?;

        parse_cspr_token_amount(value).map_err(|e| Error::raw(ErrorKind::InvalidValue, e))
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct GasParser;

impl TypedValueParser for GasParser {
    type Value = u64;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr
    ) -> Result<Self::Value, Error> {
        let parsed_value = CsprTokenAmountParser.parse_ref(cmd, arg, value)?;
        if parsed_value < U512::from(2_500_000_000u64) {
            Err(Error::raw(
                ErrorKind::ValueValidation,
                "Gas must be at least 2.5 CSPR (2,500,000,000 motes).\n"
            ))
        } else {
            Ok(parsed_value.as_u64())
        }
    }
}

#[derive(Clone)]
pub struct EnumCLParser {
    variants: Vec<(String, u16)>
}

impl EnumCLParser {
    pub fn new(variants: Vec<(String, u16)>) -> Self {
        Self { variants }
    }
}

impl TypedValueParser for EnumCLParser {
    type Value = CLValue;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr
    ) -> Result<Self::Value, Error> {
        let name = value
            .to_str()
            .ok_or_else(|| Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))?;
        let discriminant = self
            .variants
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| *d)
            .ok_or_else(|| {
                Error::raw(
                    ErrorKind::InvalidValue,
                    format!(
                        "Unknown variant '{}' for --{}. Valid: {}\n",
                        name,
                        get_arg_long(arg),
                        types::format_variant_list(&self.variants)
                    )
                )
                .with_cmd(cmd)
            })?;
        Ok(CLValue::from_components(CLType::U8, vec![discriminant as u8]))
    }
}

fn get_arg_long(arg: Option<&Arg>) -> &str {
    arg.and_then(|a| a.get_long()).unwrap_or("unknown")
}

fn make_parse_error(
    cmd: &Command,
    arg: Option<&Arg>,
    value: &str,
    ty: &NamedCLType,
    err: impl std::fmt::Display
) -> Error {
    let message = format!(
        "Failed to parse --{}\n  Value:    '{}'\n  Expected: {}\n  Cause:    {}\n",
        get_arg_long(arg),
        value,
        types::format_type_hint(ty),
        err
    );
    Error::raw(ErrorKind::InvalidValue, message).with_cmd(cmd)
}

fn parse_cspr_token_amount(value: &str) -> Result<U512, &'static str> {
    if let Ok(motes) = value.parse::<u64>() {
        return Ok(U512::from(motes));
    }

    let value = value.trim().to_lowercase();
    let value = value.strip_suffix("cspr").unwrap_or(&value).trim();

    let parts: Vec<_> = value.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid CSPR amount.");
    }

    let integer_part = parts[0]
        .parse::<u64>()
        .map(U512::from)
        .map(|v| v * 1_000_000_000)
        .map_err(|_| "Invalid CSPR amount.")?;

    if parts.len() == 1 {
        return Ok(integer_part);
    }

    let fractional_part_str = parts[1];
    if fractional_part_str.len() > 9 {
        return Err("Invalid CSPR amount: too many fractional digits.");
    }
    let fractional_part = fractional_part_str
        .parse::<u64>()
        .map(U512::from)
        .map_err(|_| "Invalid CSPR amount.")?;

    let fractional_part =
        fractional_part * U512::from(10).pow(U512::from(9 - fractional_part_str.len()));

    Ok(integer_part + fractional_part)
}

#[cfg(test)]
mod tests {
    use odra::casper_types::U512;

    use super::parse_cspr_token_amount;

    #[test]
    fn test_parse_cspr_token_amount() {
        assert_eq!(parse_cspr_token_amount("1000").unwrap(), U512::from(1000));
        assert_eq!(
            parse_cspr_token_amount("1000cspr").unwrap(),
            U512::from(1_000_000_000_000u64)
        );
        assert_eq!(
            parse_cspr_token_amount("1000 cspr").unwrap(),
            U512::from(1_000_000_000_000u64)
        );
        assert_eq!(
            parse_cspr_token_amount("2.6 cspr").unwrap(),
            U512::from(2_600_000_000u64)
        );
        assert_eq!(
            parse_cspr_token_amount("0.123456789 cspr").unwrap(),
            U512::from(123_456_789)
        );
        assert_eq!(
            parse_cspr_token_amount("0.123 cspr").unwrap(),
            U512::from(123_000_000)
        );
        assert!(
            parse_cspr_token_amount("0.1234567890 cspr").is_err(),
            "Too many fractional digits should be an error."
        );
        assert!(
            parse_cspr_token_amount("1.2.3 cspr").is_err(),
            "Multiple dots should be an error."
        );
        assert!(
            parse_cspr_token_amount("cspr").is_err(),
            "Empty amount should be an error."
        );
    }
}
