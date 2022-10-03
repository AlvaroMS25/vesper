use crate::{attr, util};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, Error, FnArg, Result, Type};

/// A command argument, and all its details, skipping the first one, which must be an `SlashContext`
/// reference.
pub struct Argument<'a> {
    /// The name of this argument at function definition.
    ///
    /// e.g.: fn a(arg: String), being `arg` this field.
    pub name: Ident,
    /// The type of this argument.
    ///
    /// e.g.: fn a(arg: String), being `String` this field.
    pub ty: Box<Type>,
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
    pub renaming: Option<String>,
    pub autocomplete: Option<Ident>,
    trait_type: &'a Type,
}

impl<'a> Argument<'a> {
    /// Creates a new [argument](self::Argument) and parses the required fields
    pub fn new(arg: FnArg, trait_type: &'a Type) -> Result<Self> {
        let pat = util::get_pat(&arg)?;
        let name = util::get_ident(&pat.pat)?;
        let type_ = pat.ty.clone();

        let mut descriptions = pat
            .attrs
            .iter()
            .map(Self::extract_description)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let mut names = pat
            .attrs
            .iter()
            .map(Self::extract_name)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let mut autocompletes = pat
            .attrs
            .iter()
            .map(Self::extract_autocomplete)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if descriptions.len() > 1 {
            // We only want a single description attribute
            return Err(Error::new(
                arg.span(),
                "Only allowed a single description attribute",
            ));
        } else if descriptions.is_empty() {
            // Description attribute is required
            return Err(Error::new(arg.span(), "Description attribute is required"));
        }

        if names.len() > 1 {
            // While this attribute is not required, we only accept a single use of it per parameter
            return Err(Error::new(
                arg.span(),
                "Only allowed a single name attribute",
            ));
        }

        if autocompletes.len() > 1 {
            return Err(Error::new(
                arg.span(),
                "Only allowed a single autocomplete attribute",
            ));
        }

        Ok(Self {
            name,
            ty: type_,
            description: descriptions.remove(0),
            renaming: if names.is_empty() {
                None
            } else {
                Some(names.remove(0))
            },
            autocomplete: if autocompletes.is_empty() {
                None
            } else {
                Some(autocompletes.remove(0))
            },
            trait_type,
        })
    }

    /// Executes the given closure into an [attr](crate::attr::Attr)
    fn exec<F, R>(attr: &Attribute, fun: F) -> Result<R>
    where
        F: FnOnce(attr::Attr) -> Result<R>,
    {
        fun(attr::parse_attribute(attr)?)
    }

    /// Extracts the description from the given attribute, returning `None` if this attribute does
    /// not correspond to the description one
    fn extract_description(attr: &Attribute) -> Result<Option<String>> {
        Self::exec(attr, |parsed| {
            if parsed.path.is_ident("description") {
                Ok(Some(parsed.parse_string()?))
            } else {
                Ok(None)
            }
        })
    }

    /// Extracts the name from a given attribute, returning `None` if this attribute does not
    /// correspond to the name one
    fn extract_name(attr: &Attribute) -> Result<Option<String>> {
        Self::exec(attr, |parsed| {
            if parsed.path.is_ident("rename") {
                Ok(Some(parsed.parse_string()?))
            } else {
                Ok(None)
            }
        })
    }

    fn extract_autocomplete(attr: &Attribute) -> Result<Option<Ident>> {
        Self::exec(attr, |parsed| {
            if parsed.path.is_ident("autocomplete") {
                if let Ok(s) = parsed.parse_string() {
                    return Ok(Some(Ident::new(&s, parsed.span())));
                }

                Ok(Some(parsed.parse_identifier()?))
            } else {
                Ok(None)
            }
        })
    }
}

impl ToTokens for Argument<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let des = &self.description;
        let ty = &self.ty;
        let tt = &self.trait_type;
        let argument_path = quote::quote!(::zephyrus::argument::CommandArgument);

        let name = match &self.renaming {
            Some(rename) => rename.clone(),
            None => self.name.to_string(),
        };

        if let Some(autocomplete) = &self.autocomplete {
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
