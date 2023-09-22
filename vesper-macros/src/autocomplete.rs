use proc_macro2::{TokenStream as TokenStream2};
use syn::{
    parse2, spanned::Spanned, Error, ItemFn, Result
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

    let data_type = util::get_context_type(&fun.sig, false)?;
    util::set_context_lifetime(&mut fun.sig)?;
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
