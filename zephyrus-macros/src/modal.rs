use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{parse2, spanned::Spanned, Block, Error, ItemFn, Result, Signature, Type, DeriveInput, Fields, FieldsNamed, Data};
use crate::{argument::Argument, details::CommandDetails, util};
use crate::attr::Attr;
use crate::optional::Optional;

struct Modal {
    title: String,
    fields: Vec<Field>
}

struct Field {
    kind: Type,
    ident: Ident,
    label: Optional<Ident>,
    placeholder: Optional<String>,
    paragraph: bool,
    max_length: Optional<u16>,
    min_length: Optional<u16>,
    value: Optional<String>
}

impl Modal {
    fn new(input: &DeriveInput, fields: FieldsNamed) -> Result<Self> {
        for attribute in input.attrs {
            let attr = crate::attr::parse_attribute(&attribute)?;

            if attr.path.get_ident().unwrap().to_string().as_str() == "title" {

            }
        }

        todo!()
    }
}

impl Field {
    fn new(field: syn::Field) -> Result<Self> {
        let mut this = Self {
            kind: field.ty.clone(),
            ident: field.ident.unwrap(),
            label: None.into(),
            placeholder: None.into(),
            paragraph: false,
            max_length: None.into(),
            min_length: None.into(),
            value: None.into()
        };


        for attribute in field.attrs {
            this.parse(crate::attr::parse_attribute(&attribute)?)
        }

        Ok(this)
    }

    fn parse(&mut self, attr: Attr) -> Result<()> {
        match attr.path.get_ident().unwrap().to_string().as_str() {
            "label" => {
                self.label = Some(attr.parse_identifier()?).into();
            },
            "placeholder" => {
                self.placeholder = Some(attr.parse_string()?).into();
            },
            "paragraph" => {
                self.paragraph = true;
            },
            "max_length" => {
                let length = attr.parse_identifier()?.to_string()
                    .parse::<u16>()
                    .expect("Not a valid number");

                self.max_length = Some(length).into();
            },
            "min_length" => {
                let length = attr.parse_identifier()?.to_string()
                    .parse::<u16>()
                    .expect("Not a valid number");

                self.min_length = Some(length).into();
            },
            "value" => {
                self.value = Some(attr.parse_string()?).into();
            }
            _ => {}
        }

        Ok(())
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {
            kind,
            ident,
            label,
            placeholder,
            paragraph,
            max_length,
            min_length,
            value
        } = &self;

        let ident = ident.to_string();
        let label = label.map(ToString::to_string).unwrap_or(ident.clone());
        let label_ref = &label;

        let style = if paragraph {
            quote::quote!(TextInputStyle::Paragraph)
        } else {
            quote::quote!(TextInputStyle::Short)
        };

        tokens.extend(quote::quote! {
            Component::ActionRow(ActionRow {
                custom_id: String::from(#label_ref),
                label: String::from(#label),
                placeholder: #placeholder,
                style: #style,
                max_length: #max_length,
                min_length: #min_length,
                required: <#kind as ModalDataOption>::required(),
                value: #value
            })
        })
    }
}

fn fields(data: Data, derive_span: impl Spanned) -> Result<FieldsNamed> {
    match data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => Ok(fields),
            Fields::Unnamed(fields) => {
                return Err(Error::new(
                    fields.span(),
                    "Unnamed fields not supported",
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

pub fn create_modal(input: TokenStream2) -> Result<TokenStream2> {
    let derive = parse2::<DeriveInput>(input)?;
    let fields = fields(derive.data, &derive)?;
    let struct_ident = &derive.ident;

    let Modal { title, fields } = Modal::new(&derive, fields);
    Ok(quote::quote! {
        const _: () = {
            use ::#crate::{
                context::SlashContext,
                extract: ModalDataOption,
                twilight_exports::{
                    InteractionResponse,
                    InteractionResponseData,
                    InteractionResponseType,
                    Component,
                    TextInput,
                    TextInputStyle
                }
            };

            #[automatically_derived]
            impl<D> ::#crate::modal::CreateModal<D> for #struct_ident {
                fn create(ctx: SlashContext<'_, D>, custom_id: String) -> InteractionResponse {
                    InteractionResponse {
                        kind: InteractionResponseType::Modal,
                        data: Some(InteractionResponseData {
                            custom_id: Some(custom_id),
                            title: Some(#title),
                            components: Some(vec![#(#fields),*])
                        })
                    }
                }
            }
        }
    })
}

pub fn parse_modal(input: TokenStream2) -> Result<TokenStream2> {
    todo!()
}
