use crate::extractors::{Either, FixedList, FunctionPath};
use crate::util;
use darling::FromMeta;
use darling::export::NestedMeta;
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{FnArg, Type};

#[derive(FromMeta)]
pub struct ArgumentAttributes {
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
    pub description: String,
    /// The renaming of this argument, if this option is not specified, the original name will be
    /// used to parse the argument and register the command in discord
    #[darling(rename = "rename")]
    pub renaming: Option<Either<String, FixedList<1, String>>>,
    pub autocomplete: Option<Either<FunctionPath, FixedList<1, FunctionPath>>>,
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
    pub attributes: ArgumentAttributes,
    trait_type: &'a Type,
}

impl<'a> Argument<'a> {
    /// Creates a new [argument](self::Argument) and parses the required fields
    pub fn new(mut arg: FnArg, trait_type: &'a Type) -> darling::Result<Self> {
        let pat = util::get_pat_mut(&mut arg)?;
        let ident = util::get_ident(&pat.pat)?;
        let ty = pat.ty.clone();

        let attributes = pat.attrs
            .drain(..)
            .map(|attribute| attribute.meta)
            .map(NestedMeta::Meta)
            .collect::<Vec<_>>();

        Ok(Self {
            ident,
            ty,
            attributes: ArgumentAttributes::from_list(attributes.as_slice())?,
            trait_type
        })
    }
}

impl ToTokens for Argument<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let des = &self.attributes.description;
        let ty = &self.ty;
        let tt = &self.trait_type;
        let argument_path = quote::quote!(::zephyrus::argument::CommandArgument);

        let name = match &self.attributes.renaming {
            Some(rename) => rename.inner().clone(),
            None => self.ident.to_string(),
        };

        if let Some(autocomplete) = &self.attributes.autocomplete {
            let autocomplete = autocomplete.clone().inner();
            tokens.extend(quote::quote! {
                .add_argument(#argument_path::<#tt>::new::<#ty>(
                    #name,
                    #des,
                    Some(#autocomplete())
                ))
            });
        } else {
            tokens.extend(quote::quote! {
                .add_argument(#argument_path::<#tt>::new::<#ty>(
                    #name,
                    #des,
                    None
                ))
            });
        }
    }
}
