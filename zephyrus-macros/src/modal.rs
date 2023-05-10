use darling::{FromDeriveInput, FromField};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{parse2, spanned::Spanned, Error, Result, Type, DeriveInput, Fields, FieldsNamed, Data};
use crate::optional::Optional;

#[derive(FromDeriveInput)]
#[darling(attributes(myderive))]
struct Modal {
    title: String,
    #[darling(skip)]
    fields: Vec<Field>
}

#[derive(FromField)]
struct Field {
    ty: Type,
    ident: Option<Ident>,
    label: Optional<String>,
    placeholder: Optional<String>,
    paragraph: bool,
    max_length: Optional<u16>,
    min_length: Optional<u16>,
    value: Optional<String>
}

struct FieldParser<'a>(&'a Field);

impl Modal {
    fn new(input: &DeriveInput, fields: &FieldsNamed) -> darling::Result<Self> {
        let mut this = Self::from_derive_input(input)?;

        for field in &fields.named {
            this.fields.push(Field::new(field)?);
        }

        Ok(this)
    }
}

impl Field {
    fn new(field: &syn::Field) -> darling::Result<Self> {
        FromField::from_field(&field)
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            ty: kind,
            ident: _,
            label,
            placeholder,
            paragraph,
            max_length,
            min_length,
            value
        } = &self;
        let label = label.as_ref().unwrap();
        let label_ref = &label;
        let placeholder = placeholder.clone().map(|p| quote::quote!(String::from(#p)));

        let style = if *paragraph {
            quote::quote!(TextInputStyle::Paragraph)
        } else {
            quote::quote!(TextInputStyle::Short)
        };

        tokens.extend(quote::quote! {
            Component::ActionRow(ActionRow {
                components: vec![
                    Component::TextInput(TextInput {
                        custom_id: String::from(#label_ref),
                        label: String::from(#label),
                        placeholder: #placeholder,
                        style: #style,
                        max_length: #max_length,
                        min_length: #min_length,
                        required: Some(<#kind as ModalDataOption>::required()),
                        value: #value
                    })
                ]
            })
        })
    }
}

impl<'a> ToTokens for FieldParser<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.0.ident;
        let label = &self.0.label.as_ref().unwrap();

        tokens.extend(quote::quote! {
            #label => {
                #ident = component.value;
            }
        })
    }
}

fn fields(data: &Data, derive_span: impl Spanned) -> Result<&FieldsNamed> {
    match data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => Ok(fields),
            Fields::Unnamed(fields) => {
                return Err(Error::new(
                    fields.span(),
                    "Tuple structs not supported",
                ))
            },
            Fields::Unit => {
                return Err(Error::new(
                    s.fields.span(),
                    "Unit structs not supported",
                ))
            }
        },
        _ => {
            return Err(Error::new(
                derive_span.span(),
                "This derive is only available for structs",
            ))
        }
    }
}

pub fn modal(input: TokenStream2) -> Result<TokenStream2> {
    let mut derive = parse2::<DeriveInput>(input)?;
    let fields = fields(&derive.data, &derive)?;

    let Modal { title, fields } = Modal::new(&derive, fields)?;
    let struct_ident = &derive.ident;

    let parsers = fields.iter()
        .map(FieldParser)
        .collect::<Vec<FieldParser>>();
    let field_names = fields.iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<&Ident>>();
    let field_types = fields.iter()
        .map(|field| &field.ty)
        .collect::<Vec<&Type>>();

    Ok(quote::quote! {
        const _: () = {
            use ::zephyrus::{
                context::SlashContext,
                extract::ModalDataOption,
                twilight_exports::{
                    Interaction,
                    InteractionData,
                    InteractionResponse,
                    InteractionResponseData,
                    InteractionResponseType,
                    ActionRow,
                    Component,
                    TextInput,
                    TextInputStyle
                }
            };

            #[automatically_derived]
            impl<D> ::zephyrus::modal::Modal<D> for #struct_ident {
                fn create(ctx: &SlashContext<'_, D>, custom_id: String) -> InteractionResponse {
                    InteractionResponse {
                        kind: InteractionResponseType::Modal,
                        data: Some(InteractionResponseData {
                            custom_id: Some(custom_id),
                            title: Some(String::from(#title)),
                            components: Some(vec![#(#fields),*]),
                            ..std::default::Default::default()
                        })
                    }
                }

                fn parse(interaction: &mut Interaction) -> Self {
                    let Some(InteractionData::ModalSubmit(modal)) = &mut interaction.data else {
                        unreachable!();
                    };

                    #(let mut #field_names = None;)*

                    let components = std::mem::take(&mut modal.components);
                    for row in components {
                        for component in row.components {
                            match component.custom_id.as_str() {
                                #(#parsers,)*
                                _ => panic!("Unrecognized field")
                            }
                        }
                    }

                    Self {
                        #(#field_names: <#field_types as ModalDataOption>::parse(#field_names)),*
                    }
                }
            }
        };
    })
}
