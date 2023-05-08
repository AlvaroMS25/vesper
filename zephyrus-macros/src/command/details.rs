use darling::FromMeta;
use darling::export::NestedMeta;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::Token;
use syn::{Attribute, Result};
use syn::punctuated::Punctuated;

#[derive(Default, FromMeta)]
#[darling(default)]
/// The details of a given command
pub struct CommandDetails {
    /// The description of this command
    pub description: String,
    pub required_permissions: Option<Punctuated<Ident, Token![,]>>,
    pub checks: Punctuated<Ident, Token![,]>,
    pub error_handler: Option<Ident>
}

impl CommandDetails {
    pub fn parse(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let meta = attrs
            .drain(..)
            .map(|item| item.meta)
            .map(NestedMeta::Meta)
            .collect::<Vec<_>>();

        Self::from_list(meta.as_slice())
            .map_err(From::from)
    }
}

impl ToTokens for CommandDetails {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let d = &self.description;
        tokens.extend(quote::quote!(.description(#d)));

        if let Some(permissions) = &self.required_permissions {
            let mut permission_stream = TokenStream2::new();

            for (index, permission) in permissions.iter().enumerate() {
                if index == 0 || permissions.len() == 1 {
                    permission_stream
                        .extend(quote::quote!(zephyrus::twilight_exports::Permissions::#permission))
                } else {
                    permission_stream.extend(
                        quote::quote!( | zephyrus::twilight_exports::Permissions::#permission),
                    )
                }
            }

            tokens.extend(quote::quote!(.required_permissions(#permission_stream)));
        }

        let checks = self.checks.iter();

        tokens.extend(quote::quote! {
            .checks(vec![#(#checks()),*])
        });

        if let Some(error_handler) = &self.error_handler {
            tokens.extend(quote::quote!(.error_handler(#error_handler())));
        }
    }
}
