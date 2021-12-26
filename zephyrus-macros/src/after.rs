use proc_macro2::TokenStream as TokenStream2;
use syn::{parse2, spanned::Spanned, Error, ItemFn, Result};

/// The implementation of after macro, this macro takes the given input, which must be another
/// function and prepares it to be an after hook, wrapping it in a struct and providing a pointer
/// to the actual function
pub fn after(input: TokenStream2) -> Result<TokenStream2> {
    let fun = parse2::<ItemFn>(input)?;
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = fun;

    match sig.inputs.len() {
        c if c != 3 => {
            // This hook is expected to have three arguments, a reference to an `SlashContext`,
            // a &str indicating the name of the command and the result of a command execution.
            return Err(Error::new(sig.inputs.span(), "Expected two arguments"));
        }
        _ => (),
    };

    // The name of the original function
    let ident = sig.ident.clone();
    // This is the name the given function will have after this macro's execution
    let fn_ident = quote::format_ident!("_{}", &ident);
    sig.ident = fn_ident.clone();

    /*
    Check the return of the function, returning if it does not match, this function is required
    to return `()`
    */
    crate::util::check_return_type(&sig.output, quote::quote!(()))?;

    let (_, ty) = crate::util::get_context_type_and_ident(&sig)?;
    // Get the futurize macro so we can fit the function into a normal fn pointer
    let futurize = crate::util::get_futurize_macro();
    let path = quote::quote!(::zephyrus::hook::AfterHook);

    Ok(quote::quote! {
        pub fn #ident() -> #path<#ty> {
            #path(#fn_ident)
        }

        #[#futurize]
        #(#attrs)*
        #vis #sig #block
    })
}
