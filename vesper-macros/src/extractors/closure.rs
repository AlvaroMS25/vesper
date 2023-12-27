use darling::{Error, FromMeta};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, ExprClosure};
use syn::parse::{Parse, ParseStream};

#[derive(Clone)]
pub struct Closure(pub ExprClosure);

impl FromMeta for Closure {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Closure(closure) => Ok(Self(closure.clone())),
            _ => Err(Error::unexpected_expr_type(expr))
        }.map_err(|e| e.with_span(expr))
    }
}

impl ToTokens for Closure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl Parse for Closure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(ExprClosure::parse(input)?))
    }
}
