use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use crate::extractors::{Either, FunctionPath};
use crate::extractors::closure::Closure;

#[derive(Clone)]
pub struct FunctionOrClosure(Either<Closure, FunctionPath>);

impl ToTokens for FunctionOrClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl FromMeta for FunctionOrClosure {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        Ok(Self(FromMeta::from_nested_meta(item)?))
    }
}

impl Parse for FunctionOrClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(Parse::parse(input)?))
    }
}
