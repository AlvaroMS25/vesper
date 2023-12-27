use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use quote::ToTokens;
use syn::{Expr, ExprPath};
use syn::parse::{Parse, ParseStream};

use super::{Either, Ident};

#[derive(Clone)]
pub struct FunctionPath(Either<Ident, ExprPath>);

impl ToTokens for FunctionPath {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl FromMeta for FunctionPath {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        Ok(Self(FromMeta::from_nested_meta(item)?))
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => Ok(Self(Either::Right(path.clone()))),
            _ => Err(Error::unexpected_expr_type(expr))
        }.map_err(|e| e.with_span(expr))
    }
}

impl Parse for FunctionPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Either::Right(Parse::parse(input)?)))
    }
}
