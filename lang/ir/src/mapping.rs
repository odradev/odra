use proc_macro2::{TokenTree, TokenStream};
use quote::ToTokens;
use syn::{bracketed, parse::ParseStream};

#[derive(Debug)]
pub struct MapExpr {
    pub root_mapping: syn::Expr,
    pub segments: Vec<syn::Expr>,
    pub assign_token: Option<syn::Token![=]>,
    pub assigned_value: Option<syn::Expr>,
}

impl syn::parse::Parse for MapExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut root_expr_stream = TokenStream::new();
        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                if matches!(tt, TokenTree::Group(_)) {
                    return Ok(((), rest));
                } else {
                    tt.to_tokens(&mut root_expr_stream);
                    rest = next;
                }
            }
            Err(cursor.error("no `TokenTree::Group` was found after this point"))
        })?;
        
        let mut segments = Vec::new();        
        while !input.is_empty() && !input.lookahead1().peek(syn::Token![=]) {
            let content;
            let _bracket_token = bracketed!(content in input);

            while !content.is_empty() {
                segments.push(content.parse()?);
            }
        }

        if segments.is_empty() {
            return Err(input.error("expected at least one segment"));
        }
        
        let root_mapping = syn::parse2(root_expr_stream)?;

        if !input.is_empty() {
            let assign_token = input.parse()?;
            let assigned_value = input.parse()?;
            Ok(Self {
                root_mapping,
                segments,
                assign_token: Some(assign_token),
                assigned_value: Some(assigned_value),
            })
        } else {
            Ok(Self {
                root_mapping,
                segments,
                assign_token: None,
                assigned_value: None,
            })
        }
    }
}


#[cfg(test)]
mod test {
    use crate::MapExpr;

    #[test]
    fn basic_parsing_works() {
        let simple_expr = syn::parse_str::<MapExpr>("self.tokens[a]").unwrap();
        
        assert_eq!(simple_expr.segments.len(), 1);
        assert_eq!(simple_expr.assign_token, None);
        assert_eq!(simple_expr.assigned_value, None);

        let complex_expr = syn::parse_str::<MapExpr>("self.tokens[a][b][c][d][e]").unwrap();

        assert_eq!(complex_expr.segments.len(), 5);
        assert_eq!(complex_expr.assign_token, None);
        assert_eq!(complex_expr.assigned_value, None);

        let invalid_expr = syn::parse_str::<MapExpr>("self.tokens[a][b"); // missing closing bracket
        assert!(invalid_expr.is_err());

        let invalid_expr = syn::parse_str::<MapExpr>("self.tokens.get(a)"); // invalid syntax
        assert!(invalid_expr.is_err());

        let invalid_expr = syn::parse_str::<MapExpr>("self.tokens(a)(b)"); // parenthesis found, brackets expected
        assert!(invalid_expr.is_err());

        let invalid_expr = syn::parse_str::<MapExpr>("self.tokens"); // no brackets found
        assert!(invalid_expr.is_err());
    }

    #[test]
    fn assigning_parsing_works() {
        let simple_expr = syn::parse_str::<MapExpr>("self.tokens[a] = 1").unwrap();
        
        assert_eq!(simple_expr.segments.len(), 1);
        assert!(simple_expr.assign_token.is_some());
        assert!(simple_expr.assigned_value.is_some());

        let complex_expr = syn::parse_str::<MapExpr>("self.tokens[a][b][c][d][e] = String::from(3)").unwrap();

        assert_eq!(complex_expr.segments.len(), 5);
        assert!(complex_expr.assign_token.is_some());
        assert!(complex_expr.assigned_value.is_some());
    }

    #[test]
    fn parsing_complex_expressions_works() {
        let simple_expr = syn::parse_str::<MapExpr>("get_mapping[self.build_key()][String::from(1)] = calculate_value()").unwrap();
        
        assert_eq!(simple_expr.segments.len(), 2);
        assert!(simple_expr.assign_token.is_some());
        assert!(simple_expr.assigned_value.is_some());
    }
}