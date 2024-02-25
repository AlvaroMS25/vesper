use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{spanned::Spanned, DeriveInput, Error, Result};
use crate::extractors::{Either, FixedList};

#[derive(FromAttributes)]
#[darling(attributes(parse))]
struct VariantAttributes {
    #[darling(rename = "rename")]
    renaming: Option<Either<String, FixedList<1, String>>>,
}

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
            choices.push(CommandOptionChoice {
                    name: #value.to_string(),
                    name_localizations: None,
                    value: CommandOptionChoiceValue::Integer(#index)
                }
            );
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

        let attributes = VariantAttributes::from_attributes(variant.attrs.as_slice())?;

        let name = attributes.renaming
            .map(|item| item.inner().clone())
            .unwrap_or(variant.ident.to_string());

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
            use ::vesper::{
                builder::WrappedClient,
                prelude::async_trait,
                parse::{Parse, ParseError},
                twilight_exports::{
                    InteractionDataResolved,
                    CommandOptionChoice,
                    CommandOptionChoiceValue,
                    CommandOptionType,
                    CommandOptionValue,

                },
            };

            #[automatically_derived]
            #[async_trait]
            impl<T: Send + Sync + 'static> Parse<T> for #enum_name {
                async fn parse(
                    http_client: &WrappedClient,
                    data: &T,
                    value: Option<&CommandOptionValue>,
                    resolved: Option<&mut InteractionDataResolved>
                ) -> Result<Self, ParseError>
                {
                    let num = usize::parse(http_client, data, value, resolved).await?;
                    match num {
                        #parse_stream
                        _ => return Err(ParseError::Parsing {
                                argument_name: String::new(),
                                required: true,
                                argument_type: String::from(stringify!(#enum_name)),
                                error: String::from("Unrecognized option")
                            }
                        )
                    }
                }
                fn kind() -> CommandOptionType {
                    CommandOptionType::Integer
                }
                fn choices() -> Option<Vec<CommandOptionChoice>> {
                    let mut choices = Vec::new();

                    #choice_stream;

                    Some(choices)
                }
            }
        };
    })
}
