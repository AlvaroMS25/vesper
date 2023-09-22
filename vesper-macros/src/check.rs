use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, spanned::Spanned, Error, ItemFn, Result};
use crate::util;

pub fn check(input: TokenStream2) -> Result<TokenStream2> {
    let fun = parse2::<ItemFn>(input)?;
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = fun;

    if sig.inputs.len() > 1 {
        // This hook is expected to have a single `&SlashContext` parameter.
        return Err(Error::new(
            sig.inputs.span(),
            "Function parameter must only be &SlashContext",
        ));
    }

    // The name of the original macro
    let ident = sig.ident.clone();
    // The name the function will have after this macro's execution
    let fn_ident = quote::format_ident!("_{}", &ident);
    sig.ident = fn_ident.clone();

    let return_type = util::get_return_type(&sig)?;
    let returnable = util::get_returnable_trait();

    let ty = util::get_context_type(&sig, true)?;
    // Get the hook macro so we can fit the function into a normal fn pointer
    let hook = util::get_hook_macro();
    let path = quote::quote!(::zephyrus::hook::CheckHook);

    Ok(quote::quote! {
        pub fn #ident() -> #path<#ty, <#return_type as #returnable>::Err> {
            #path(#fn_ident)
        }

        #[#hook]
        #(#attrs)*
        #vis #sig #block
    })
}
