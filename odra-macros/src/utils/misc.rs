use syn::parse_quote;

pub fn ret_ty(ty: &syn::Type) -> syn::ReturnType {
    parse_quote!(-> #ty)
}

pub trait AsBlock {
    fn as_block(&self) -> syn::Block;
}

impl<T: quote::ToTokens> AsBlock for T {
    fn as_block(&self) -> syn::Block {
        parse_quote!({ #self })
    }
}

pub trait AsExpr {
    fn as_expr(&self) -> ::syn::Expr;
}

impl<T: quote::ToTokens> AsExpr for T {
    fn as_expr(&self) -> ::syn::Expr {
        ::syn::parse_quote!(#self)
    }
}
pub trait AsStmt {
    fn as_stmt(&self) -> ::syn::Stmt;
}

impl<T: quote::ToTokens> AsStmt for T {
    fn as_stmt(&self) -> ::syn::Stmt {
        ::syn::parse_quote!(#self)
    }
}
