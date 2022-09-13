use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse2, spanned::Spanned, Error, GenericArgument, ItemFn, Lifetime, PathArguments, Result,
    Signature, Type,
};

pub fn autocomplete(input: TokenStream2) -> Result<TokenStream2> {
    let mut fun = parse2::<ItemFn>(input)?;

    if fun.sig.inputs.len() != 1 {
        return Err(Error::new(
            fun.sig.inputs.span(),
            "Autocomplete hook must have as parameters an AutocompleteContext<D>",
        ));
    }

    let data_type = get_data_type_and_set_lifetime(&fun.sig)?;
    set_lifetime(&mut fun.sig)?;
    let futurize = crate::util::get_futurize_macro();
    let path = quote::quote!(::zephyrus::hook::AutocompleteHook);
    let ident = fun.sig.ident.clone();
    let fn_ident = quote::format_ident!("_{}", ident);
    fun.sig.ident = fn_ident.clone();

    Ok(quote::quote! {
        pub fn #ident() -> #path<#data_type> {
            #path(#fn_ident)
        }

        #[#futurize]
        #fun
    })
}

fn get_data_type_and_set_lifetime(sig: &Signature) -> Result<Type> {
    let ctx = match sig.inputs.iter().next() {
        None => {
            return Err(Error::new(
                sig.inputs.span(),
                "Expected AutocompleteContext as only paramenter",
            ))
        }
        Some(c) => crate::util::get_pat(c)?,
    };
    let mut generics = crate::util::get_generic_arguments(crate::util::get_path(&ctx.ty)?)?;

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

fn set_lifetime(sig: &mut Signature) -> Result<()> {
    let lifetime = Lifetime::new("'future", Span::call_site());
    let ctx = crate::util::get_pat_mut(sig.inputs.iter_mut().next().unwrap())?;
    let path = crate::util::get_path_mut(&mut ctx.ty)?;
    let mut insert_lifetime = true;

    {
        let generics = crate::util::get_generic_arguments(path)?;
        for generic in generics {
            if let GenericArgument::Lifetime(inner) = generic {
                if *inner == lifetime {
                    insert_lifetime = false;
                }
            }
        }
    }

    if insert_lifetime {
        if let PathArguments::AngleBracketed(inner) =
            &mut path.segments.last_mut().unwrap().arguments
        {
            inner.args.insert(0, GenericArgument::Lifetime(lifetime));
        }
    }

    Ok(())
}
