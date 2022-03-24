use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, spanned::Spanned, Error, ItemFn, Result};

pub fn autocomplete(input: TokenStream2) -> Result<TokenStream2> {
    let mut fun = parse2::<ItemFn>(input)?;

    if fun.sig.inputs.len() != 3 {
        return Err(Error::new(
            fun.sig.inputs.span(),
            "Autocomplete hook must have as parameters a &WrappedClient, a reference to the data and an ApplicationCommandAutocomplete"
        ));
    }

    let futurize = crate::util::get_futurize_macro();
    let path = quote::quote!(::zephyrus::hook::BeforeHook);
    let ident = fun.sig.ident.clone();
    let fn_ident = quote::format_ident!("_{}", ident);
    fun.sig.ident = fn_ident.clone();

    Ok(quote::quote! {
        pub fn #ident<D>() -> #path<D> {
            #path(#fn_ident)
        }

        #[#futurize]
        #fun
    })
}
