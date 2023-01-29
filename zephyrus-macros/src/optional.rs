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
