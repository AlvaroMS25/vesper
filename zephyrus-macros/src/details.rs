use crate::attr::*;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::ToTokens;
use std::convert::TryFrom;
use syn::{spanned::Spanned, Attribute, Error, Result};

#[derive(Default)]
/// The details of a given command
pub struct CommandDetails {
    /// The description of this command
    pub description: String,
    pub required_permissions: Option<Vec<Ident>>,
    pub checks: Vec<Ident>
}

impl CommandDetails {
    pub fn parse(attrs: &mut Vec<Attribute>) -> Result<Self> {
        let mut s = Self::default();

        let i = 0;

        while i < attrs.len() {
            let attr = &attrs[i];

            match attr.path.get_ident().unwrap().to_string().as_str() {
                "description" => {
                    if !s.description.is_empty() {
                        return Err(Error::new(attr.span(), "Description already set"));
                    }

                    s.description = {
                        let a = Attr::try_from(attr)?;
                        a.parse_string()?
                    };
                }
                "required_permissions" => {
                    let a = Attr::try_from(attr)?;
                    let permissions = a.parse_all()?;
                    s.required_permissions = Some(permissions);
                },
                "checks" => {
                    let attr = Attr::try_from(attr)?;
                    let checks = attr.parse_all()?;
                    s.checks = checks;
                }
                _ => return Err(Error::new(attr.span(), "Attribute not recognized")),
            }

            attrs.remove(i);
        }

        if s.description.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "Description is required",
            ));
        }

        Ok(s)
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

        let checks = &self.checks;

        tokens.extend(quote::quote! {
            .checks(vec![#(#checks),*])
        });
    }
}
