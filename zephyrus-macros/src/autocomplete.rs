use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, spanned::Spanned, Error, FnArg, ItemFn, Result, Signature, Type};

pub fn autocomplete(input: TokenStream2) -> Result<TokenStream2> {
    let mut fun = parse2::<ItemFn>(input)?;

    if fun.sig.inputs.len() != 3 {
        return Err(Error::new(
            fun.sig.inputs.span(),
            "Autocomplete hook must have as parameters a &WrappedClient, a reference to the data and an ApplicationCommandAutocomplete"
        ));
    }

    let data_type = get_data_type(&fun.sig)?;
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

fn get_data_type(sig: &Signature) -> Result<Type> {
    let arg = &sig.inputs[1];

    match arg {
        FnArg::Receiver(_) => Err(Error::new(arg.span(), "`self` not allowed here")),
        FnArg::Typed(type_) => match &*type_.ty {
            Type::Reference(reference) => Ok(*reference.elem.clone()),
            _ => Err(Error::new(arg.span(), "Reference expected")),
        },
    }
}
