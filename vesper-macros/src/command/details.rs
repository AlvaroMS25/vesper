use darling::{FromMeta, export::NestedMeta};
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Token, Meta, parse2, Error, LitStr};
use syn::parse::Parse;
use syn::{Attribute, Result};
use syn::punctuated::Punctuated;

use crate::extractors::{Either, FixedList, FunctionPath, Ident, List, Map};
use crate::extractors::function_closure::FunctionOrClosure;

#[derive(Default, FromMeta)]
/// The details of a given command
pub struct CommandDetails {
    #[darling(skip, default)]
    pub input_options: InputOptions,
    #[darling(default)]
    pub localized_names: Option<Map<LitStr, LitStr>>,
    /// The description of this command
    pub description: Either<String, FixedList<1, String>>,
    #[darling(default)]
    pub localized_descriptions: Option<Either<FunctionOrClosure, Map<LitStr, LitStr>>>,
    #[darling(default)]
    pub required_permissions: Option<List<Ident>>,
    #[darling(default)]
    pub checks: Either<List<FunctionPath>, Punctuated<FunctionPath, Token![,]>>,
    #[darling(default)]
    pub error_handler: Option<Either<FunctionPath, FixedList<1, FunctionPath>>>,
    #[darling(default)]
    pub nsfw: bool,
    #[darling(default)]
    pub only_guilds: bool
}

impl CommandDetails {
    pub fn parse(input_options: InputOptions, attrs: &mut Vec<Attribute>) -> Result<Self> {
        let meta = attrs
            .drain(..)
            .map(|item| item.meta)
            .map(NestedMeta::Meta)
            .collect::<Vec<_>>();

        let mut this = Self::from_list(meta.as_slice())?;

        this.input_options = input_options;
        Ok(this)
    }
}

impl ToTokens for CommandDetails {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let options = &self.input_options;
        tokens.extend(quote::quote!(#options));

        if let Some(localized_names) = &self.localized_names {
            let localized_names = localized_names.pairs();
            tokens.extend(quote::quote!(.localized_names(vec![#(#localized_names),*])))
        }

        let d = self.description.inner();
        tokens.extend(quote::quote!(.description(#d)));
        
        if let Some(localized_descriptions) = &self.localized_descriptions {
            match localized_descriptions {
                Either::Right(map) => {
                    let localized_descriptions = map.pairs();
                    tokens.extend(quote::quote!(.localized_descriptions(vec![#(#localized_descriptions),*])))
                },
                Either::Left(function) => {

                }
            }
        }

        if let Some(permissions) = &self.required_permissions {
            let mut permission_stream = TokenStream2::new();

            for (index, permission) in permissions.iter().enumerate() {
                if index == 0 || permissions.len() == 1 {
                    permission_stream
                        .extend(quote::quote!(vesper::twilight_exports::Permissions::#permission))
                } else {
                    permission_stream.extend(
                        quote::quote!( | vesper::twilight_exports::Permissions::#permission),
                    )
                }
            }

            tokens.extend(quote::quote!(.required_permissions(#permission_stream)));
        }

        let mut checks = Vec::new();
        self.checks.map_1(
            &mut checks,
            |checks, a| checks.extend(a.iter().cloned()),
            |checks, b| checks.extend(b.iter().cloned())
        );

        tokens.extend(quote::quote! {
            .checks(vec![#(#checks()),*])
        });

        if let Some(error_handler) = &self.error_handler {
            let error_handler = error_handler.inner();
            tokens.extend(quote::quote!(.error_handler(#error_handler())));
        }

        let nsfw = self.nsfw;
        let only_guilds = self.only_guilds;

        tokens.extend(quote::quote!(
            .nsfw(#nsfw)
            .only_guilds(#only_guilds)
        ));
    }
}

#[derive(Default, FromMeta)]
pub struct InputOptions {
    #[darling(default)]
    pub chat: bool,
    #[darling(default)]
    pub message: bool,
    #[darling(default)]
    pub user: bool,
    #[darling(default)]
    pub name: String
}

impl InputOptions {
    pub fn new(stream: TokenStream2, ident: &syn::Ident) -> Result<Self> {
        let stream_empty = stream.is_empty();
        let stream_clone = stream.clone();
        let span = stream.span();
        let meta = match parse2::<MetaListParser>(stream) {
            Ok(m) => m.0,
            Err(_) if !stream_empty => {
                return Ok(Self {
                    chat: true,
                    name: parse2::<syn::LitStr>(stream_clone)?.value(),
                    ..Default::default()
                })
            },
            Err(e) => return Err(e)
        };

        let meta = meta.into_iter()
            .map(NestedMeta::Meta)
            .collect::<Vec<_>>();

        let mut this = Self::from_list(&meta)?;

        if !(this.chat || this.message || this.user) {
            this.chat = true;
        }

        if this.name.is_empty() {
            this.name = ident.to_string();
        }

        if !(this.chat ^ this.message ^ this.user) || (this.chat && this.message && this.user) {
            return Err(Error::new(
                span,
                "Only one of `chat`, `message` or `user` can be selected"
            ));
        }

        Ok(this)
    }
}

impl ToTokens for InputOptions {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        tokens.extend(quote::quote!(.name(#name)));

        if self.chat {
            return;
        }


        let kind = if self.user {
            quote::quote!(::vesper::twilight_exports::CommandType::User)
        } else if self.message {
            quote::quote!(::vesper::twilight_exports::CommandType::Message)
        } else {
            unreachable!()
        };

        tokens.extend(quote::quote!(.kind(#kind)));
    }
}

pub struct MetaListParser(pub Punctuated<Meta, Token![,]>);

impl Parse for MetaListParser {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self(input.call(Punctuated::parse_terminated)?))
    }
}
