use crate::attr::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use std::convert::TryInto;
use syn::{spanned::Spanned, Attribute, Error, Result};

#[derive(Default)]
/// The details of a given command
pub struct CommandDetails {
    /// The description of this command
    pub description: String,
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
                        let a: &Attr = &attr.try_into()?;
                        a.parse_string()?
                    };
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
    }
}
