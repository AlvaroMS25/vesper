use std::marker::PhantomData;

use darling::{FromMeta, export::NestedMeta};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{Token, parse::{Parse, ParseStream}};

pub struct Tuple2<K, V, D = Token![=]>(pub K, pub V, PhantomData<D>);

impl<K, V, D> Tuple2<K, V, D> {
    pub fn new(k: K, v: V) -> Self {
        Self(k, v, PhantomData)
    }
}

impl<K, V, D> Parse for Tuple2<K, V, D>
where
    K: Parse,
    V: Parse,
    D: Parse
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let k = input.parse()?;
        let _: D = input.parse()?;
        let v = input.parse()?;

        Ok(Self::new(k, v))
    }
}

impl<K, V, D> FromMeta for Tuple2<K, V, D> 
where
    K: FromMeta,
    V: FromMeta,
{
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        match items {
            [a, b] => 
                Ok(Self(K::from_nested_meta(a)?, V::from_nested_meta(b)?, PhantomData)),
            _ => Err(darling::Error::unsupported_format("2 item tuple").with_span(&Span::call_site()))
        }
    }
}

impl<K, V, D> ToTokens for Tuple2<K, V, D>
where
    K: ToTokens,
    V: ToTokens
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let k = &self.0;
        let v = &self.1;
        tokens.extend(quote::quote!((#k, #v)))
    }
}
