use quote::{quote, ToTokens};
use syn::parse_quote;

pub struct LetStmtBuilder {
    mut_token: Option<syn::Token![mut]>,
    name: syn::Ident,
    value: Option<syn::Expr>,
}

impl LetStmtBuilder {
    pub fn new(name: &syn::Ident) -> Self {
        Self {
            mut_token: None,
            name: name.clone(),
            value: None,
        }
    }

    pub fn set_mut(mut self) -> Self {
        self.mut_token = Some(Default::default());
        self
    }

    pub fn value(mut self, value: syn::Expr) -> Self {
        self.value = Some(value);
        self
    }

    pub fn build(self) -> syn::Stmt {
        let name = self.name;
        let value = self.value.unwrap_or_else(|| syn::parse_quote!(Default::default()));
        syn::parse_quote!(let #name = #value;)
    }
}

pub struct FnCallBuilder {
    caller: Option<syn::Expr>,
    fn_ident: syn::Ident,
    args: Vec<syn::Expr>
}

impl FnCallBuilder {
    pub fn new(fn_ident: syn::Ident) -> Self {
        Self {
            caller: None,
            fn_ident,
            args: Vec::new()
        }
    }

    pub fn caller<T: ToTokens>(mut self, caller: T) -> Self {
        let caller = caller.into_token_stream();
        self.caller = Some(parse_quote!(#caller));
        self
    }

    pub fn arg<T: ToTokens>(mut self, arg: T) -> Self {
        let arg = arg.into_token_stream();
        self.args.push(parse_quote!(#arg));
        self
    }

    pub fn build(self) -> syn::Expr {
        let caller = self.caller.map(|caller| quote!(#caller.));
        let fn_ident = self.fn_ident;
        let args = self.args;
        parse_quote!(#caller #fn_ident(#(#args),*))
    }
}

pub struct MatchBuilder {
    expr: syn::Expr,
    arms: Vec<syn::Arm>
}

impl MatchBuilder {
    pub fn new(expr: syn::Expr) -> Self {
        Self {
            expr,
            arms: Vec::new()
        }
    }

    pub fn arm<T: ToTokens>(mut self, pat: T, expr: syn::Expr) -> Self {
        let pat = pat.into_token_stream();
        self.arms.push(parse_quote!(#pat => #expr));
        self
    }

    pub fn arms<T: ToTokens>(mut self, arms: Vec<T>) -> Self {
        let arms = arms
            .into_iter()
            .map(|arm| arm.into_token_stream())
            .map(|arm| parse_quote!(#arm))
            .collect::<Vec<_>>();
        self.arms.extend(arms);
        self
    }

    pub fn build(self) -> syn::Expr {
        let expr = self.expr;
        let arms = self.arms;
        parse_quote!(match #expr { #(#arms),* })
    }
}

pub struct StructBuilder {
    struct_ty: syn::Type,
    fields: Vec<syn::FieldValue>
}

impl StructBuilder {
    pub fn new(struct_ty: syn::Type) -> Self {
        Self {
            struct_ty,
            fields: Vec::new()
        }
    }

    pub fn field<T: ToTokens>(mut self, ident: syn::Ident, value: T) -> Self {
        let value = value.into_token_stream();
        self.fields.push(parse_quote!(#ident: #value));
        self
    }

    pub fn fields<T: ToTokens>(mut self, fields: Vec<(syn::Ident, T)>) -> Self {
        let fields = fields.iter()
            .map(|(ident, value)| {
                let value = value.into_token_stream();
                parse_quote!(#ident: #value)
            })
            .collect::<Vec<_>>();
        self.fields.extend(fields);
        self
    }

    pub fn build(self) -> syn::Expr {
        let struct_ty = self.struct_ty;
        let fields = self.fields;
        parse_quote!(#struct_ty { #(#fields),* })
    }
}