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

impl ToTokens for MapExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let root = &self.root_mapping;
        let init_root = quote::quote!(let v = &#root;);
        let segments_count = self.segments.len();
        
        let value = self.segments.last().unwrap();

        if self.assign_token.is_some() && self.assigned_value.is_some() {
            let assigned_value = self.assigned_value.as_ref().unwrap();
            let dest_mapping_discovery = self.segments
                .iter()
                .take(segments_count - 1)
                .rev()
                .enumerate()
                .map(|(idx, e)| match idx {
                    0 => quote::quote!(let mut v = v.get_instance(&#e);),
                    _ => quote::quote!(let v = v.get_instance(&#e);),
                })
                .rev()
                .collect::<TokenStream>();
            let value_assign = quote::quote!(v.set(&#value, #assigned_value));
            
            quote::quote! {
                #init_root
                #dest_mapping_discovery
                #value_assign;
            }.to_tokens(tokens);
        } else {
            let value_discovery = self.segments
                .iter()
                .take(segments_count - 1)
                .map(|e| quote::quote!(let v = v.get_instance(&#e);))
                .collect::<TokenStream>();
            let return_value = quote::quote!(odra::UnwrapOrRevert::unwrap_or_revert(v.get(&#value)));
            quote::quote! {
                {
                    #init_root
                    #value_discovery
                    #return_value
                }
            }.to_tokens(tokens);
        }
    }
}


#[cfg(test)]
mod test {
    use quote::ToTokens;

    use crate::map::MapExpr;

    #[test]
    fn parsing_works() {
        let expr: MapExpr = syn::parse_str("self.tokens[b][c]").unwrap();
        
        // dbg!(expr);
        // assert_eq!(expr.expr.to_token_stream().to_string(), "a");
        // assert_eq!(expr.segments.len(), 2);
        dbg!(expr.to_token_stream().to_string());
        // assert_eq!(expr.segments[1].1.to_token_stream().to_string(), "c");
        assert!(false);
    }
}
