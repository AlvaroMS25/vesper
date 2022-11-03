use proc_macro2::{TokenStream as TokenStream2};
use syn::{
    parse2, spanned::Spanned, Error, GenericArgument, ItemFn, Result,
    Signature, Type,
};
use crate::util;

pub fn autocomplete(input: TokenStream2) -> Result<TokenStream2> {
    let mut fun = parse2::<ItemFn>(input)?;

    if fun.sig.inputs.len() != 1 {
        return Err(Error::new(
            fun.sig.inputs.span(),
            "Autocomplete hook must have as parameters an AutocompleteContext<D>",
        ));
    }

    let data_type = get_data_type_and_set_lifetime(&fun.sig)?;
    util::set_lifetimes(&mut fun.sig)?;
    let hook = util::get_hook_macro();
    let path = quote::quote!(::zephyrus::hook::AutocompleteHook);
    let ident = fun.sig.ident.clone();
    let fn_ident = quote::format_ident!("_{}", ident);
    fun.sig.ident = fn_ident.clone();

    Ok(quote::quote! {
        pub fn #ident() -> #path<#data_type> {
            #path(#fn_ident)
        }

        #[#hook]
        #fun
    })
}

fn get_data_type_and_set_lifetime(sig: &Signature) -> Result<Type> {
    let ctx = match sig.inputs.iter().next() {
        None => {
            return Err(Error::new(
                sig.inputs.span(),
                "Expected AutocompleteContext as only parameter",
            ))
        }
        Some(c) => util::get_pat(c)?,
    };
    let mut generics = util::get_generic_arguments(crate::util::get_path(&ctx.ty, true)?)?;

    let ty = loop {
        match generics.next() {
            Some(GenericArgument::Lifetime(_)) => (),
            Some(a) => {
                break match a {
                    GenericArgument::Type(t) => {
                        if let Type::Infer(_) = t {
                            return Err(Error::new(
                                sig.inputs.span(),
                                "AutocompleteContext must have a known type",
                            ));
                        } else {
                            t.clone()
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            sig.inputs.span(),
                            "AutocompleteContext type must be a type",
                        ))
                    }
                }
            }
            None => {
                return Err(Error::new(
                    sig.inputs.span(),
                    "AutocompleteContext type must be set",
                ))
            }
        }
    };

    Ok(ty)
}
