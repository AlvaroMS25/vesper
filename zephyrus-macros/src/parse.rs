use crate::attr::Attr;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use std::convert::TryFrom;
use syn::{spanned::Spanned, DeriveInput, Error, Result};

struct Variant {
    value: String,
    ident: Ident,
    index: usize,
}

impl Variant {
    fn parse_tokens(&self, tokens: &mut TokenStream2) {
        let index = &self.index;
        let ident = &self.ident;
        tokens.extend(quote::quote! {
            #index => Ok(Self::#ident),
        })
    }

    fn choice_tokens(&self, tokens: &mut TokenStream2) {
        let value = &self.value;
        let index = self.index as i64;
        tokens.extend(quote::quote! {
            choices.push(::zephyrus::twilight_exports::CommandOptionChoice::Int {
                name: #value.to_string(),
                value: #index,
                name_localizations: None
            });
        })
    }
}

pub fn parse(input: TokenStream2) -> Result<TokenStream2> {
    let derive = syn::parse2::<DeriveInput>(input)?;
    let enumeration = match derive.data {
        syn::Data::Enum(e) => e,
        _ => {
            return Err(Error::new(
                derive.ident.span(),
                "This derive is only available for enums",
            ))
        }
    };

    let mut variants = Vec::new();
    let mut index = 1;

    for variant in enumeration.variants {
        if !matches!(&variant.fields, syn::Fields::Unit) {
            return Err(Error::new(
                variant.span(),
                "Choice parameter cannot have inner values",
            ));
        }

        let mut name = variant.ident.to_string();
        for attribute in variant.attrs {
            let attr = Attr::try_from(&attribute)?;
            if attr.path.is_ident("rename") {
                name = attr.parse_string()?;
            }
        }

        variants.push(Variant {
            ident: variant.ident.clone(),
            value: name,
            index,
        });

        index += 1;
    }

    let mut parse_stream = TokenStream2::new();
    let mut choice_stream = TokenStream2::new();
    for variant in variants {
        variant.parse_tokens(&mut parse_stream);
        variant.choice_tokens(&mut choice_stream);
    }

    let enum_name = &derive.ident;

    Ok(quote::quote! {
        const _: () = {
            #[automatically_derived]
            #[::zephyrus::prelude::async_trait]
            impl<T: Send + Sync + 'static> ::zephyrus::prelude::Parse<T> for #enum_name {
                async fn parse(
                    http_client: &::zephyrus::builder::WrappedClient,
                    data: &T,
                    value: Option<&::zephyrus::twilight_exports::CommandOptionValue>,
                ) -> Result<Self, ::zephyrus::prelude::ParseError>
                {
                    let num = usize::parse(http_client, data, value).await?;
                    match num {
                        #parse_stream
                        _ => return Err(::zephyrus::parse::ParseError::Parsing {
                                argument_name: String::new(),
                                required: true,
                                type_: String::from(stringify!(#enum_name)),
                                error: String::from("Unrecognized option")
                            }
                        )
                    }
                }
                fn kind() -> ::zephyrus::twilight_exports::CommandOptionType {
                    ::zephyrus::twilight_exports::CommandOptionType::Integer
                }
                fn choices() -> Option<Vec<::zephyrus::twilight_exports::CommandOptionChoice>> {
                    let mut choices = Vec::new();

                    #choice_stream;

                    Some(choices)
                }
            }
        };
    })
}
