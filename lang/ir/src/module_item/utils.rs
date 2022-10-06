use syn::parse_quote;

pub fn payable_check(impl_block: syn::Block, is_payable: bool) -> syn::Block {
    let attached_value_check: Option<syn::Stmt> = match is_payable {
        true => None,
        false => Some(parse_quote!(if odra::ContractEnv::attached_value()
            > odra::types::U512::zero()
        {
            odra::ContractEnv::revert(odra::types::ExecutionError::non_payable());
        })),
    };
    let mut stmts = impl_block.stmts;
    // add payable check as the first function statement.
    if let Some(check) = attached_value_check {
        stmts.insert(0, check);
    }

    parse_quote!({
        #(#stmts)*
    })
}

#[cfg(test)]
mod test {
    use quote::ToTokens;
    use syn::parse_quote;

    use super::payable_check;

    fn block() -> syn::Block {
        parse_quote!(if value > 0 { 42 } else { 1 })
    }

    #[test]
    fn test_payable() {
        let expected: syn::Block = parse_quote!({ 42 });

        let result = payable_check(block(), true);

        assert_eq!(
            expected.to_token_stream().to_string(),
            result.to_token_stream().to_string()
        );
    }

    #[test]
    fn test_non_payable() {
        let expected: syn::Block = parse_quote!({
            if odra::ContractEnv::attached_value() > odra::types::U256::zero() {
                odra::ContractEnv::revert(odra::types::ExecutionError::non_payable());
            }
            42
        });

        let result = payable_check(block(), false);

        assert_eq!(
            expected.to_token_stream().to_string(),
            result.to_token_stream().to_string()
        );
    }
}
