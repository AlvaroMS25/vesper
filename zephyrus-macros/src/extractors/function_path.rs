use darling::ast::NestedMeta;
use darling::FromMeta;
use quote::ToTokens;
use syn::ExprPath;
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
}

impl Parse for FunctionPath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Either::Right(Parse::parse(input)?)))
    }
}
