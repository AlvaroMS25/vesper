use darling::FromMeta;
use quote::ToTokens;
use syn::Meta;

#[derive(Clone)]
pub struct Ident(syn::Ident);

impl FromMeta for Ident {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        match item {
            Meta::Path(p) if p.get_ident().is_some() => Ok(Self(p.get_ident().unwrap().clone())),
            _ => Err(darling::Error::custom("Expected identifier"))
        }
    }

    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        Ok(Self(<syn::Ident as FromMeta>::from_value(value)?))
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self(<syn::Ident as FromMeta>::from_string(value)?))
    }
}

impl ToTokens for Ident {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        <syn::Ident as ToTokens>::to_tokens(&self.0, tokens)
    }
}
