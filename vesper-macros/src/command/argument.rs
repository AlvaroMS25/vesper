use crate::extractors::{Either, FixedList, FunctionPath, Map};
use crate::optional::Optional;
use crate::util;
use darling::FromMeta;
use darling::export::NestedMeta;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{FnArg, Type, LitStr, Error};

#[derive(FromMeta)]
pub struct ArgumentAttributes {
    #[darling(default)]
    pub localized_names: Option<Map<LitStr, LitStr>>,
    /// The description of this argument, this is a required field parsed with `#[description]`
    /// attribute.
    ///
    /// This macro can be used two ways:
    ///
    ///     - List way: #[description("Some description")]
    ///
    ///     - Named value way: #[description = "Some description"]
    ///
    /// e.g.: fn a(#[description = "some here"] arg: String), being the fields inside `description`
    /// this field
    #[darling(default)]
    pub description: Option<Either<String, FixedList<1, String>>>,
    #[darling(default)]
    pub localized_descriptions: Option<Map<LitStr, LitStr>>,
    /// The renaming of this argument, if this option is not specified, the original name will be
    /// used to parse the argument and register the command in discord
    #[darling(rename = "rename")]
    pub renaming: Option<Either<String, FixedList<1, String>>>,
    pub autocomplete: Optional<Either<FunctionPath, FixedList<1, FunctionPath>>>,
    #[darling(default)]
    pub skip: bool
}

/// A command argument, and all its details, skipping the first one, which must be an `SlashContext`
/// reference.
pub struct Argument<'a> {
    /// The name of this argument at function definition.
    ///
    /// e.g.: fn a(arg: String), being `arg` this field.
    pub ident: Ident,
    /// The type of this argument.
    ///
    /// e.g.: fn a(arg: String), being `String` this field.
    pub ty: Box<Type>,
    /// Argument attributes, only present if the command is a `chat` command.
    pub attributes: Option<ArgumentAttributes>,
    trait_type: &'a Type,
    pub chat_command: bool
}

impl<'a> Argument<'a> {
    /// Creates a new [argument](self::Argument) and parses the required fields
    pub fn new(mut arg: FnArg, trait_type: &'a Type, chat_command: bool) -> darling::Result<Self> {
        let pat = util::get_pat_mut(&mut arg)?;
        let ident = util::get_ident(&pat.pat)?;
        let ty = pat.ty.clone();

        let attributes = pat.attrs
            .drain(..)
            .map(|attribute| attribute.meta)
            .map(NestedMeta::Meta)
            .collect::<Vec<_>>();

        let this = Self {
            ident,
            ty,
            attributes: if chat_command {
                Some(ArgumentAttributes::from_list(attributes.as_slice())?)
            } else {
                None
            },
            trait_type,
            chat_command
        };

        if chat_command 
            && !this.attributes.as_ref().map(|a| a.skip).unwrap_or(false)
            && this.attributes.as_ref().unwrap()
                .description.as_ref().map(|d| d.inner().is_empty()).unwrap_or(true) 
        {
            return Err(Error::new(
                arg.span(),
                "Missing `description`"
            )).map_err(From::from);
        }

        Ok(this)
    }
}

impl ToTokens for Argument<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.attributes.as_ref().map(|a| a.skip).unwrap_or(false) || !self.chat_command {
            return;
        }
        let attributes = self.attributes.as_ref().unwrap();

        let des = attributes.description.as_ref().unwrap().inner();
        let ty = &self.ty;
        let tt = &self.trait_type;
        let argument_path = quote::quote!(::zephyrus::argument::CommandArgument);

        let name = match &attributes.renaming {
            Some(rename) => rename.inner().clone(),
            None => self.ident.to_string(),
        };

        let add_localized_names = attributes.localized_names.as_ref().map(|map| {
            let localized_names = map.pairs();
            quote::quote!(.localized_names(vec![#(#localized_names),*]))
        });

        let add_localized_descriptions = attributes.localized_descriptions.as_ref().map(|map| {
            let localized_descriptions = map.pairs();
            quote::quote!(.localized_descriptions(vec![#(#localized_descriptions),*]))
        });

        let autocomplete = attributes.autocomplete.as_ref().map(|either| {
            let inner = either.inner();
            quote::quote!(#inner())
        });

        tokens.extend(quote::quote! {
            .add_argument(#argument_path::<#tt>::new::<#ty>(
                #name,
                #des,
                #autocomplete
            )
            #add_localized_names
            #add_localized_descriptions
            )   
        });
    }
}
