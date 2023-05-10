use std::ops::{Deref, DerefMut};
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Default)]
pub struct Optional<T>(Option<T>);

impl<T: ToTokens> ToTokens for Optional<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(inner) = &self.0 {
            tokens.extend(quote::quote!(Some(#inner)));
        } else {
            tokens.extend(quote::quote!(None))
        }
    }
}

impl<T: ToTokens> From<Option<T>> for Optional<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T: FromMeta> FromMeta for Optional<T> {
    fn from_nested_meta(item: &darling::export::NestedMeta) -> darling::Result<Self> {
        Ok(Self(FromMeta::from_nested_meta(item)?))
    }
}

impl<T> Deref for Optional<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Optional<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone> Clone for Optional<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Optional<T> {
    pub fn map<F, R>(self, fun: F) -> Optional<R>
    where
        F: FnOnce(T) -> R
    {
        Optional(self.0.map(fun))
    }
}
